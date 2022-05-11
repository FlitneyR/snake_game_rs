use pixel_canvas::{
    Canvas,
    canvas::CanvasInfo,
    input::Event,
    input::glutin::event::VirtualKeyCode,
    input::WindowEvent, Color,
};

fn main() {
    let canvas = Canvas::new(500, 500)
        .state(GameState::new())
        .title("Snake")
        .input(GameState::input_handler);

    canvas.render(|state, image| {
        let cells: isize = (image.width() / GameState::CELLSIZE) as isize;

        if state.framecount % 5 != 0 {
            state.framecount += 1;
            return;
        }

        state.update(&cells);

        let width = image.width();

        for (y, row) in image.chunks_mut(width).enumerate() {
            for (x, pixel) in row.into_iter().enumerate() {
                *pixel = background();

                if state.point_in_snake(x, y) {
                    *pixel = snake_color();
                }
            }
        }

        state.framecount += 1;
    });
}

fn background() -> Color {
    Color {
        r: 0,
        g: 0,
        b: 0,
    }
}

fn snake_color() -> Color {
    Color {
        r: 255,
        g: 255,
        b: 255,
    }
}

struct GameState {
    framecount: usize,
    head_x: usize,
    head_y: usize,
    head_dx: isize,
    head_dy: isize,
    to_grow: isize,
    body: Vec<(usize, usize)>
}

impl GameState {
    const CELLSIZE: usize = 15;

    fn new() -> Self {
        Self {
            framecount: 0,
            head_x: 5,
            head_y: 5,
            head_dx: 1,
            head_dy: 0,
            to_grow: 5,
            body: vec![],
        }
    }

    fn update(&mut self, cells: &isize) {
        self.head_x = ((self.head_x as isize + self.head_dx + cells) % cells) as usize;
        self.head_y = ((self.head_y as isize + self.head_dy + cells) % cells) as usize;

        self.body.push((self.head_x, self.head_y));
        self.to_grow -= 1;

        if self.to_grow < 0 {
            self.body.remove(0);
            self.to_grow += 1;
        }
    }

    fn point_in_snake(&self, x: usize, y: usize) -> bool {
        for (px, py) in self.body.iter() {
            let sx = px * Self::CELLSIZE;
            let sy = py * Self::CELLSIZE;

            let ex = sx + Self::CELLSIZE;
            let ey = sy + Self::CELLSIZE;

            if (sx..=ex).contains(&x) && (sy..=ey).contains(&y) {
                return true
            }
        }
        false
    }

    fn input_handler(_info: &CanvasInfo, state: &mut Self, event: &Event<()>) -> bool {
        match event {
            Event::WindowEvent {
                event: window_event,
                ..
            } => {
                match window_event {
                    WindowEvent::KeyboardInput {
                        input,
                        ..
                    } => {
                        (state.head_dx, state.head_dy) = match input.virtual_keycode {
                            Some(VirtualKeyCode::W) => ( 0,  1),
                            Some(VirtualKeyCode::S) => ( 0, -1),
                            Some(VirtualKeyCode::A) => (-1,  0),
                            Some(VirtualKeyCode::D) => ( 1,  0),
                            _ => (state.head_dx, state.head_dy),
                        };
                    },
                    _ => (),
                }
            },
            _ => (),
        }

        true
    }
}
