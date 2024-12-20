use picofb::{DrawHandle, Event, Framebuffer, Key, MainLoop};

struct State {
    time: u64,
}

impl MainLoop for State {
    fn handle_event(&mut self, fb: &mut Framebuffer, event: &Event) {
        if matches!(event, Event::KeyPress(Key::Escape)) {
            fb.close();
        }
    }

    fn update(&mut self, _fb: &mut Framebuffer, _dt: f32, time: f64) {
        self.time = time as u64;
    }

    fn render(&mut self, d: &mut DrawHandle) {
        d.clear();
        d.draw_text(20, 20, 0xff_ff_ff, "Hello, world!");
        d.draw_text(20, 40, 0xff_ff_ff, &format!("Current time: {}", self.time));
    }
}

fn main() {
    let mut fb = Framebuffer::new(300, 300, "Text rendering", 30);
    let mut state = State { time: 0 };

    fb.run(&mut state);
}
