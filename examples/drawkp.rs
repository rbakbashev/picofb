use picofb::{DrawHandle, Event, Framebuffer, Key, MainLoop};

struct State;

impl MainLoop for State {
    fn handle_event(&mut self, fb: &mut Framebuffer, event: &Event) {
        if matches!(event, Event::KeyPress(Key::Escape)) {
            fb.close();
        }
    }

    fn update(&mut self, _fb: &mut Framebuffer, _dt: f32, _time: f64) {}

    fn render(&mut self, d: &mut DrawHandle) {
        let color = if d.key_pressed(Key::Space) {
            0x770000
        } else {
            0x007700
        };

        d.as_slice().fill(color);
    }
}

fn main() {
    let mut state = State;
    let mut fb = Framebuffer::new(300, 300, "Space to change color", 60);

    fb.run(&mut state);
}
