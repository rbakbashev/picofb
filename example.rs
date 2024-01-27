use picofb::{DrawHandle, Event, Framebuffer, Key, MainLoop};

#[derive(Default)]
struct MyGameState {
    pos_x: f64,
    pos_y: f64,
    input_forward: i8,
    input_right: i8,
}

impl MainLoop for MyGameState {
    fn handle_event(&mut self, fb: &mut Framebuffer, event: &Event) {
        match event {
            Event::KeyPress(key) => match key {
                Key::W => self.input_forward = 1,
                Key::S => self.input_forward = -1,
                Key::D => self.input_right = 1,
                Key::A => self.input_right = -1,
                Key::Escape => fb.close(),
                _ => (),
            },
            Event::KeyRelease(key) => match key {
                Key::W if self.input_forward == 1 => self.input_forward = 0,
                Key::S if self.input_forward == -1 => self.input_forward = 0,
                Key::D if self.input_right == 1 => self.input_right = 0,
                Key::A if self.input_right == -1 => self.input_right = 0,
                _ => (),
            },
        }
    }

    fn update(&mut self, fb: &mut Framebuffer, dt: f64, _time: f64) {
        if self.input_forward == 0 && self.input_right == 0 {
            return;
        }

        self.pos_x += f64::from(self.input_right) * 40.0 * dt;
        self.pos_y -= f64::from(self.input_forward) * 40.0 * dt;

        self.pos_x = self.pos_x.clamp(0.0, fb.width().into());
        self.pos_y = self.pos_y.clamp(0.0, fb.height().into());
    }

    fn render(&mut self, d: &mut DrawHandle) {
        d.clear();

        let size = 10;

        let px = self.pos_x as u32;
        let py = self.pos_y as u32;

        let xmax = (px + size).min(d.width());
        let ymax = (py + size).min(d.height());

        for y in py..ymax {
            for x in px..xmax {
                d.set(x, y, 0xff0000);
            }
        }
    }
}

fn main() {
    let mut state = MyGameState::default();
    let mut fb = Framebuffer::new(300, 300, "example", 16);

    fb.run(&mut state);
}
