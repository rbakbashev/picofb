use picofb::{DrawHandle, Event, Framebuffer, Key, MainLoop};

#[derive(Default)]
struct MyGameState {
    pos_x: f32,
    pos_y: f32,
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
                Key::W => self.input_forward = 0,
                Key::S => self.input_forward = 0,
                Key::D => self.input_right = 0,
                Key::A => self.input_right = 0,
                _ => (),
            },
            Event::MouseMove(_, _) => (),
        }
    }

    fn update(&mut self, fb: &mut Framebuffer, dt: f32, _time: f64) {
        self.pos_x += 50. * dt * f32::from(self.input_right);
        self.pos_y -= 50. * dt * f32::from(self.input_forward);

        self.pos_x = self.pos_x.clamp(0., fb.width() as f32);
        self.pos_y = self.pos_y.clamp(0., fb.height() as f32);
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
    let mut fb = Framebuffer::new(300, 300, "example", 60);

    fb.run(&mut state);
}
