use picofb::{Event, Framebuffer, MainLoop};

struct MyGameState {
    pos_x: u32,
    pos_y: u32,
    input_forward: i8,
    input_right: i8,
}

impl MyGameState {
    fn new() -> Self {
        Self {
            pos_x: 0,
            pos_y: 0,
            input_forward: 0,
            input_right: 0,
        }
    }
}

impl MainLoop for MyGameState {
    fn handle_event(&mut self, event: &Event) {
        println!("event={:?}", event);
    }

    fn update(&mut self, _dt: f64, _time: f64) {}

    fn render(&mut self, fb: &mut Framebuffer) {
        fb.set(10, 30, 0xff0000);
    }
}

fn main() {
    let mut state = MyGameState::new();
    let mut fb = Framebuffer::new(800, 600, "example", 16);

    fb.run(&mut state);
}
