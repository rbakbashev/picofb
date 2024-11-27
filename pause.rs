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
        d.clear();

        d.pause(Key::C);

        draw_circle(d, 70, 150, 30, 0x00cc00);
        d.pause(Key::C);

        draw_circle(d, 150, 150, 30, 0x00cc00);
        d.pause(Key::C);

        draw_circle(d, 230, 150, 30, 0x00cc00);
        d.pause(Key::C);
    }
}

fn draw_circle(d: &mut DrawHandle, x: u32, y: u32, radius: u32, color: u32) {
    let x_min = x.saturating_sub(radius + 1);
    let y_min = y.saturating_sub(radius + 1);

    let x_max = x.saturating_add(radius).min(d.width() - 1);
    let y_max = y.saturating_add(radius).min(d.height() - 1);

    for dx in x_min..x_max {
        for dy in y_min..y_max {
            let x_off = (dx as i32) - (x as i32) + 1;
            let y_off = (dy as i32) - (y as i32) + 1;
            let rad_sq = (radius * radius) as i32;

            if x_off * x_off + y_off * y_off <= rad_sq {
                d.set(dx, dy, color);
            }
        }
    }
}

fn main() {
    let mut state = State;
    let mut fb = Framebuffer::new(300, 300, "C to continue", 60);

    fb.run(&mut state);
}
