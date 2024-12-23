#![allow(clippy::missing_const_for_fn, clippy::must_use_candidate)]

pub mod key;
mod text_renderer;

use std::collections::HashMap;
use std::ffi::{c_int, CStr, CString};
use std::mem::{size_of, ManuallyDrop, MaybeUninit};
use std::ptr;
use std::slice;

#[allow(clippy::wildcard_imports)]
use sdl2_sys::*;

pub use key::Key;
use text_renderer::TextRenderer;

pub struct Framebuffer {
    main_window: ManuallyDrop<Window>,
    running: bool,
    dt: f32,
    fps_buf: FpsCounter,
}

struct Window {
    width: u32,
    height: u32,
    handle: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
    texture: *mut SDL_Texture,
    _id: u32,
    key_pressed: HashMap<Key, bool>,
    title: &'static str,
}

pub struct DrawHandle<'p, 'w> {
    width: u32,
    height: u32,
    pixels: &'p mut [u32],
    window: &'w Window,
}

#[derive(Debug)]
pub enum Event {
    KeyPress(Key),
    KeyRelease(Key),
    MouseMove(i32, i32),
}

pub trait MainLoop {
    fn handle_event(&mut self, fb: &mut Framebuffer, event: &Event);
    fn update(&mut self, fb: &mut Framebuffer, dt: f32, time: f64);
    fn render(&mut self, d: &mut DrawHandle);
}

trait CheckErr {
    fn check_err(self, action: &'static str) -> Self;
}

struct FpsCounter {
    measurements: Vec<f64>,
    idx: usize,
    sum: f64,
}

const TEXT_RENDERER: TextRenderer = TextRenderer::new();

impl Framebuffer {
    pub fn new(width: u32, height: u32, title: &'static str, update_rate: i16) -> Self {
        init_library();

        let main_window = ManuallyDrop::new(Window::new(width, height, title));
        let running = true;
        let dt = 1. / f32::from(update_rate);
        let fps_buf = FpsCounter::new(32);

        Self {
            main_window,
            running,
            dt,
            fps_buf,
        }
    }

    fn poll_events(&mut self, state: &mut impl MainLoop) {
        let mut event_ptr = MaybeUninit::<SDL_Event>::uninit();

        loop {
            unsafe {
                if SDL_PollEvent(event_ptr.as_mut_ptr()) == 0 {
                    break;
                }

                let event = event_ptr.assume_init();
                let type_ = std::mem::transmute::<u32, SDL_EventType>(event.type_);

                match type_ {
                    SDL_EventType::SDL_KEYDOWN => {
                        let key = std::mem::transmute::<i32, Key>(event.key.keysym.sym);
                        let event = Event::KeyPress(key);
                        self.main_window
                            .key_pressed
                            .entry(key)
                            .and_modify(|pr| *pr = true)
                            .or_insert(true);
                        state.handle_event(self, &event);
                    }
                    SDL_EventType::SDL_KEYUP => {
                        let key = std::mem::transmute::<i32, Key>(event.key.keysym.sym);
                        let event = Event::KeyRelease(key);
                        self.main_window
                            .key_pressed
                            .entry(key)
                            .and_modify(|pr| *pr = false)
                            .or_insert(false);
                        state.handle_event(self, &event);
                    }
                    SDL_EventType::SDL_MOUSEMOTION => {
                        let event = Event::MouseMove(event.motion.xrel, event.motion.yrel);
                        state.handle_event(self, &event);
                    }
                    SDL_EventType::SDL_QUIT => self.running = false,
                    _ => (),
                }
            }
        }
    }

    fn show_fps(&mut self, real_time: f64) {
        let elapsed = current_time_seconds() - real_time;
        let average = self.fps_buf.add_measurement(1. / elapsed);

        self.set_window_title(&format!("{} FPS {:5.3}", self.main_window.title, average));
    }

    pub fn run(&mut self, state: &mut impl MainLoop) {
        let mut current_time = current_time_seconds();

        while self.running {
            let real_time = current_time_seconds();

            while current_time < real_time {
                current_time += f64::from(self.dt);

                self.poll_events(state);
                state.update(self, self.dt, current_time);
            }

            if !self.running {
                break;
            }

            let mut handle = self.main_window.start_render();
            state.render(&mut handle);
            self.main_window.present();

            limit_fps(500.0, real_time);
            self.show_fps(real_time);
        }
    }

    pub fn benchmark(&mut self, state: &mut impl MainLoop, frames: usize) {
        let title = self.main_window.title;
        let mut current_time = current_time_seconds();
        let mut frame = 0;

        while self.running && frame < frames {
            let real_time = current_time_seconds();

            while current_time < real_time {
                current_time += f64::from(self.dt);

                self.poll_events(state);
                state.update(self, self.dt, current_time);
            }

            if !self.running {
                break;
            }

            let mut handle = self.main_window.start_render();
            state.render(&mut handle);
            self.main_window.present();

            limit_fps(500.0, real_time);

            frame += 1;

            self.set_window_title(&format!("{} frame {}/{}", title, frame, frames));
        }
    }

    pub fn width(&self) -> u32 {
        self.main_window.width
    }

    pub fn height(&self) -> u32 {
        self.main_window.height
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn widthf(&self) -> f32 {
        self.main_window.width as f32
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn heightf(&self) -> f32 {
        self.main_window.height as f32
    }

    pub fn close(&mut self) {
        self.running = false;
    }

    pub fn set_window_title(&mut self, title: &str) {
        self.main_window.set_window_title(title);
    }

    pub fn grab_mouse(&mut self, enabled: bool) {
        set_mouse_grab(enabled);
    }

    pub fn grab_state(&mut self) -> bool {
        get_mouse_grab()
    }

    pub fn mouse_pos(&self) -> (i32, i32) {
        let mut x = 0;
        let mut y = 0;

        unsafe {
            SDL_GetMouseState(&mut x, &mut y);
        }

        (x, y)
    }

    pub fn mouse_pos_rel(&self) -> (i32, i32) {
        let mut x = 0;
        let mut y = 0;

        unsafe {
            SDL_GetRelativeMouseState(&mut x, &mut y);
        }

        (x, y)
    }

    pub fn key_pressed(&self, key: Key) -> bool {
        self.main_window.key_pressed(key)
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.main_window);
            SDL_Quit();
        }
    }
}

impl Window {
    fn new(width: u32, height: u32, title: &'static str) -> Self {
        let w_int = width as c_int;
        let h_int = height as c_int;
        let handle = create_window(w_int, h_int, title);
        let renderer = create_renderer(handle);
        let texture = create_texture(renderer, w_int, h_int);
        let id = get_window_id(handle);
        let key_pressed = HashMap::with_capacity(240);

        Self {
            width,
            height,
            handle,
            renderer,
            texture,
            _id: id,
            key_pressed,
            title,
        }
    }

    fn start_render(&self) -> DrawHandle {
        let mut ptr: *mut u32 = ptr::null_mut();
        let mut pitch = 0;
        let num_pixels = (self.width * self.height) as usize;

        let pixels = unsafe {
            SDL_LockTexture(
                self.texture,
                ptr::null(),
                ptr::addr_of_mut!(ptr).cast(),
                &mut pitch,
            );

            debug_assert!(pitch / self.width as i32 == size_of::<u32>() as i32);

            slice::from_raw_parts_mut(ptr, num_pixels)
        };

        DrawHandle {
            width: self.width,
            height: self.height,
            pixels,
            window: self,
        }
    }

    fn present(&self) {
        unsafe {
            SDL_UnlockTexture(self.texture);
            SDL_RenderCopy(self.renderer, self.texture, ptr::null(), ptr::null());
            SDL_RenderPresent(self.renderer);
        }
    }

    fn key_pressed(&self, key: Key) -> bool {
        *self.key_pressed.get(&key).unwrap_or(&false)
    }

    fn set_window_title(&self, title: &str) {
        let cstr = CString::new(title).expect("Title contains null byte");

        unsafe {
            SDL_SetWindowTitle(self.handle, cstr.as_ptr());
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            SDL_DestroyTexture(self.texture);
            SDL_DestroyRenderer(self.renderer);
            SDL_DestroyWindow(self.handle);
        }
    }
}

impl<'p, 'w> DrawHandle<'p, 'w> {
    pub fn clear(&mut self) {
        self.pixels.fill(0);
    }

    pub fn set(&mut self, x: u32, y: u32, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }

        let idx = y * self.width + x;
        self.pixels[idx as usize] = color | 0xff_00_00_00;
    }

    pub unsafe fn set_unchecked(&mut self, x: u32, y: u32, color: u32) {
        let idx = y * self.width + x;
        self.set_unchecked_index(idx as usize, color);
    }

    unsafe fn set_unchecked_index(&mut self, idx: usize, color: u32) {
        *self.pixels.get_unchecked_mut(idx) = color;
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn widthf(&self) -> f32 {
        self.width as f32
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn heightf(&self) -> f32 {
        self.height as f32
    }

    pub fn as_slice(&mut self) -> &mut [u32] {
        self.pixels
    }

    pub fn pause(&mut self, unpause_key: Key) {
        let grab = get_mouse_grab();

        set_mouse_grab(false);
        self.window
            .set_window_title(&format!("{} [paused]", self.window.title));

        while !poll_key_pressed(unpause_key) {
            self.window.present();
            unsafe { SDL_Delay(16) };
        }

        set_mouse_grab(grab);
    }

    pub fn key_pressed(&self, key: Key) -> bool {
        self.window.key_pressed(key)
    }

    pub fn draw_text(&mut self, pos_x: u32, pos_y: u32, color: u32, text: &str) {
        TEXT_RENDERER.render(self, pos_x, pos_y, color, text);
    }
}

impl CheckErr for c_int {
    fn check_err(self, action: &'static str) -> Self {
        if self == 0 {
            return self;
        }

        let err_str = unsafe { CStr::from_ptr(SDL_GetError()) };

        panic!("Failed to {action}: {err_str:?}");
    }
}

impl<T> CheckErr for *mut T {
    fn check_err(self, action: &'static str) -> Self {
        if !self.is_null() {
            return self;
        }

        let err_str = unsafe { CStr::from_ptr(SDL_GetError()) };

        panic!("Failed to {action}: {err_str:?}");
    }
}

impl FpsCounter {
    pub fn new(num_measurements: usize) -> Self {
        Self {
            measurements: vec![0.; num_measurements],
            idx: 0,
            sum: 0.,
        }
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn add_measurement(&mut self, fps: f64) -> f64 {
        let num_measurements = self.measurements.len();

        self.sum -= self.measurements[self.idx];
        self.sum += fps;
        self.measurements[self.idx] = fps;
        self.idx += 1;
        self.idx %= num_measurements;

        self.sum / (num_measurements as f64)
    }
}

fn init_library() {
    let flags = SDL_INIT_VIDEO | SDL_INIT_EVENTS | SDL_INIT_TIMER;

    unsafe { SDL_Init(flags) }.check_err("initialize SDL");
}

fn create_window(w: c_int, h: c_int, title: &'static str) -> *mut SDL_Window {
    let cstr = CString::new(title).expect("Title contains null byte");
    let any_pos = SDL_WINDOWPOS_UNDEFINED_MASK as c_int;
    let flags = 0;

    unsafe { SDL_CreateWindow(cstr.as_ptr(), any_pos, any_pos, w, h, flags) }
        .check_err("create window")
}

fn create_renderer(window: *mut SDL_Window) -> *mut SDL_Renderer {
    let flags = SDL_RendererFlags::SDL_RENDERER_ACCELERATED as u32;

    unsafe { SDL_CreateRenderer(window, -1, flags) }.check_err("create renderer")
}

fn create_texture(renderer: *mut SDL_Renderer, w: c_int, h: c_int) -> *mut SDL_Texture {
    let format = SDL_PixelFormatEnum::SDL_PIXELFORMAT_ARGB8888 as u32;
    let access = SDL_TextureAccess::SDL_TEXTUREACCESS_STREAMING as c_int;

    unsafe { SDL_CreateTexture(renderer, format, access, w, h) }.check_err("create texture")
}

fn get_window_id(window: *mut SDL_Window) -> u32 {
    unsafe { SDL_GetWindowID(window) }
}

fn current_time_seconds() -> f64 {
    let ms = unsafe { SDL_GetTicks() };

    f64::from(ms) / 1000.0
}

fn limit_fps(target_fps: f64, real_time: f64) {
    let frame_end = current_time_seconds();
    let frame_time = frame_end - real_time;
    let to_sleep_f = 1000.0 / target_fps - frame_time;

    if to_sleep_f.is_sign_negative() {
        return;
    }

    let to_sleep = to_sleep_f.floor() as u32;

    unsafe { SDL_Delay(to_sleep) };
}

fn poll_key_pressed(key: Key) -> bool {
    let mut event_ptr = MaybeUninit::<SDL_Event>::uninit();

    loop {
        unsafe {
            if SDL_PollEvent(event_ptr.as_mut_ptr()) == 0 {
                break;
            }

            let event = event_ptr.assume_init();
            let type_ = std::mem::transmute::<u32, SDL_EventType>(event.type_);

            match type_ {
                SDL_EventType::SDL_KEYDOWN => {
                    let event_key = std::mem::transmute::<i32, Key>(event.key.keysym.sym);
                    if event_key == key {
                        return true;
                    }
                    if event_key == Key::Escape {
                        std::process::exit(0);
                    }
                }
                SDL_EventType::SDL_QUIT => std::process::exit(0),
                _ => (),
            }
        }
    }

    false
}

fn set_mouse_grab(enabled: bool) {
    let enabled = if enabled {
        SDL_bool::SDL_TRUE
    } else {
        SDL_bool::SDL_FALSE
    };

    unsafe {
        SDL_SetRelativeMouseMode(enabled);
    }
}

fn get_mouse_grab() -> bool {
    let state = unsafe { SDL_GetRelativeMouseMode() };

    state == SDL_bool::SDL_TRUE
}
