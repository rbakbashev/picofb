use picofb::{DrawHandle, Event, Framebuffer, Key, MainLoop, Window};

struct State1 {
    color: u32,
    window2: Window,
}

struct State2 {
    color: u32,
}

impl MainLoop for State1 {
    fn handle_event(&mut self, fb: &mut Framebuffer, event: &Event) {
        if matches!(event, Event::KeyPress(Key::Escape)) {
            fb.close();
        }
    }

    fn update(&mut self, _fb: &mut Framebuffer, _dt: f32, _time: f64) {}

    fn render(&mut self, d: &mut DrawHandle) {
        d.as_slice().fill(self.color);
        d.draw_text(20, 20, 0xff_ff_ff, "Hello from window 1");

        let mut state2 = State2 { color: 0x003300 };

        d.render_window(&mut self.window2, &mut state2);
    }
}

impl MainLoop for State2 {
    fn handle_event(&mut self, _fb: &mut Framebuffer, _event: &Event) {}

    fn update(&mut self, _fb: &mut Framebuffer, _dt: f32, _time: f64) {}

    fn render(&mut self, d: &mut DrawHandle) {
        d.as_slice().fill(self.color);
        d.draw_text(20, 20, 0xff_ff_ff, "Hello from window 2");
    }
}

fn main() {
    let mut fb = Framebuffer::new(300, 200, "Window 1", 30);
    let window2 = fb.add_window(200, 300, "Window 2");

    let mut state1 = State1 {
        color: 0x330000,
        window2,
    };

    fb.run(&mut state1);
}
