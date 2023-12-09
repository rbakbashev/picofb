pub mod key;

pub use crate::key::Key;

use std::ffi::{c_int, c_void, CStr, CString};
use std::mem::{size_of, MaybeUninit};
use std::ptr;
use std::slice;

#[allow(clippy::wildcard_imports)]
use sdl2_sys::*;

pub struct Framebuffer {
    width: u32,
    height: u32,
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
    texture: *mut SDL_Texture,
    running: bool,
    dt: f64,
}

pub struct DrawHandle<'p> {
    width: u32,
    height: u32,
    pixels: &'p mut [u32],
}

#[derive(Debug)]
pub enum Event {
    KeyPress(Key),
    KeyRelease(Key),
}

pub trait MainLoop {
    fn handle_event(&mut self, event: &Event);
    fn update(&mut self, dt: f64, time: f64);
    fn render(&mut self, fb: &mut DrawHandle);
}

trait CheckErr {
    fn check_err(self, action: &'static str) -> Self;
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, title: &'static str, update_rate: i16) -> Self {
        let w_int = width as c_int;
        let h_int = height as c_int;

        let window = Self::create_window(w_int, h_int, title);
        let renderer = Self::create_renderer(window);
        let texture = Self::create_texture(renderer, w_int, h_int);

        let dt = 1.0 / f64::from(update_rate);

        Self {
            width,
            height,
            window,
            renderer,
            texture,
            running: true,
            dt,
        }
    }

    fn create_window(w: c_int, h: c_int, title: &'static str) -> *mut SDL_Window {
        let cstr = CString::new(title).expect("Title contains null byte");
        let any_pos = SDL_WINDOWPOS_UNDEFINED_MASK as i32;
        let flags = SDL_WindowFlags::SDL_WINDOW_SHOWN as u32;

        unsafe { SDL_CreateWindow(cstr.as_ptr(), any_pos, any_pos, w, h, flags) }
            .check_err("create window")
    }

    fn create_renderer(window: *mut SDL_Window) -> *mut SDL_Renderer {
        let flags = SDL_RendererFlags::SDL_RENDERER_ACCELERATED as u32;

        unsafe { SDL_CreateRenderer(window, -1, flags) }.check_err("Create renderer")
    }

    fn create_texture(renderer: *mut SDL_Renderer, w: c_int, h: c_int) -> *mut SDL_Texture {
        let format = SDL_PixelFormatEnum::SDL_PIXELFORMAT_ARGB8888 as u32;
        let access = SDL_TextureAccess::SDL_TEXTUREACCESS_STREAMING as i32;

        unsafe { SDL_CreateTexture(renderer, format, access, w, h) }.check_err("create texture")
    }

    fn start_render(&mut self) -> DrawHandle {
        let mut ptr: *mut u32 = ptr::null_mut();
        let mut pitch = 0;
        let num_pixels = (self.width * self.height) as usize;

        let pixels = unsafe {
            SDL_LockTexture(
                self.texture,
                ptr::null(),
                ptr::addr_of_mut!(ptr).cast::<*mut c_void>(),
                &mut pitch,
            );

            debug_assert!(pitch / self.width as i32 == size_of::<u32>() as i32);

            slice::from_raw_parts_mut(ptr, num_pixels)
        };

        DrawHandle {
            width: self.width,
            height: self.height,
            pixels,
        }
    }

    fn present(&self) {
        unsafe {
            SDL_UnlockTexture(self.texture);
            SDL_RenderCopy(self.renderer, self.texture, ptr::null(), ptr::null());
            SDL_RenderPresent(self.renderer);
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

                let type_: SDL_EventType = std::mem::transmute(event.type_);

                match type_ {
                    SDL_EventType::SDL_KEYDOWN => {
                        let key = std::mem::transmute(event.key.keysym.sym);
                        let event = Event::KeyPress(key);
                        state.handle_event(&event);
                    }
                    SDL_EventType::SDL_KEYUP => {
                        let key = std::mem::transmute(event.key.keysym.sym);
                        let event = Event::KeyRelease(key);
                        state.handle_event(&event);
                    }
                    SDL_EventType::SDL_QUIT => self.close(),
                    _ => (),
                }
            }
        }
    }

    fn current_time_seconds() -> f64 {
        let ms = unsafe { SDL_GetTicks() };
        f64::from(ms) / 1000.0
    }

    pub fn run(&mut self, state: &mut impl MainLoop) {
        let mut current_time = Self::current_time_seconds();

        while self.running {
            self.poll_events(state);

            let real_time = Self::current_time_seconds();

            while current_time < real_time {
                current_time += self.dt;

                state.update(self.dt, current_time);
            }

            if !self.running {
                break;
            }

            let mut handle = self.start_render();
            state.render(&mut handle);
            self.present();
        }
    }

    pub fn close(&mut self) {
        self.running = false;
    }

    pub fn set_window_title(&mut self, title: &str) {
        let cstr = CString::new(title).expect("Title contains null byte");

        unsafe {
            SDL_SetWindowTitle(self.window, cstr.as_ptr());
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            SDL_DestroyTexture(self.texture);
            SDL_DestroyRenderer(self.renderer);
            SDL_DestroyWindow(self.window);
            SDL_Quit();
        }
    }
}

impl<'p> DrawHandle<'p> {
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
