use picofb::{Event, Framebuffer};

fn main() {
    let mut fb = Framebuffer::new(800, 600, "example", 16, handle_event, update, render);

    fb.run();
}

fn handle_event(event: &Event) {
    println!("event={:?}", event);
}

fn update(_dt: f64, _time: f64) {}

fn render(fb: &mut Framebuffer) {
    fb.set(10, 30, 0xff0000);
}
