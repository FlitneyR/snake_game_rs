use std::time::Instant;
use rand::Rng;

use pixel_canvas::{
    Canvas,
    canvas::CanvasInfo,
    input::{
        Event,
        glutin::event::{VirtualKeyCode, ElementState},
        WindowEvent
    },
    Color,
    Image,
};

fn main() {
    let length = GameState::CELLCOUNT * GameState::CELLSIZE;

    let canvas = Canvas::new(length, length)
        .state(GameState::new())
        .title("Snake")
        .input(GameState::input_handler);

    canvas.render(|state, image| {
        state.draw(image);

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

fn fruit_color() -> Color {
    Color {
        r: 255,
        g: 0,
        b: 0,
    }
}

struct GameState {
    gameover: bool,
    framecount: usize,
    head_x: usize,
    head_y: usize,
    head_dx: isize,
    head_dy: isize,
    to_grow: isize,
    body: Vec<(usize, usize)>,
    fruit_x: isize,
    fruit_y: isize,
    last_tick: Instant,
    move_queue: Vec<(isize, isize)>,
    should_clear: bool,
    erase_last: bool,
}

impl GameState {
    const CELLSIZE: usize = 15;
    const CELLCOUNT: usize = 30;

    fn new() -> Self {
        Self {
            gameover: false,
            framecount: 0,
            head_x: 5,
            head_y: 5,
            head_dx: 1,
            head_dy: 0,
            to_grow: 5,
            body: vec![],
            fruit_x: -1,
            fruit_y: -1,
            last_tick: Instant::now(),
            move_queue: vec![],
            should_clear: true,
            erase_last: true,
        }
    }

    fn update(&mut self, cells: &isize) {
        match self.move_queue.get(0) {
            Some((dx, dy)) => {
                if self.head_dx == -*dx && self.head_dy == -*dy {
                    ()
                } else {
                    self.head_dx = *dx;
                    self.head_dy = *dy;
                }
                self.move_queue.remove(0);
            },
            _ => ()
        }

        self.head_x = ((self.head_x as isize + self.head_dx + cells) % cells) as usize;
        self.head_y = ((self.head_y as isize + self.head_dy + cells) % cells) as usize;

        match (
            self.fruit_x.try_into() as Result<usize, _>,
            self.fruit_y.try_into() as Result<usize, _>
        ) {
            (Ok(fx), Ok(fy)) => {
                if fx == self.head_x && fy == self.head_y {
                    self.fruit_x = -1;
                    self.fruit_y = -1;
                    self.to_grow += 1;
                }
            },
            _ => {
                let mut rng = rand::thread_rng();

                self.fruit_x = (rng.gen::<usize>() % GameState::CELLCOUNT) as isize;
                self.fruit_y = (rng.gen::<usize>() % GameState::CELLCOUNT) as isize;
            }
        }

        for (px, py) in self.body.iter() {
            if *px == self.head_x && *py == self.head_y {
                self.gameover = true;
            }
        }

        self.body.push((self.head_x, self.head_y));
        self.to_grow -= 1;

        if self.erase_last {
            self.body.remove(0);
            self.erase_last = false;
        }

        if self.to_grow < 0 {
            self.to_grow += 1;
            self.erase_last = true;
        }
    }

    fn _erase(&self, image: &mut Image) {
        let w = Self::CELLSIZE;
        let h = w;
        let col = background();

        for (px, py) in self.body.iter() {
            draw_square(image, *px * w, *py * h, w, h, col);
        }
    }

    fn draw(&mut self, image: &mut Image) {
        if self.last_tick.elapsed().as_millis() < 100 {
            return;
        }

        if self.gameover {
            return
        }

        if self.should_clear {
            draw_square(
                image, 0, 0,
                image.width(), image.height(),
                background()
            );

            self.should_clear = false;
        }

        self.last_tick = Instant::now();

        let cells: isize = (image.width() / GameState::CELLSIZE) as isize;

        let w = Self::CELLSIZE;
        let h = w;
        let col = background();

        if self.erase_last {
            match self.body.get(0) {
                Some((x, y)) => draw_square(image, *x * w, *y * h, w, h, col),
                _ => ()
            }
        }

        self.update(&cells);

        let col = snake_color();

        match self.body.last() {
            Some((x, y)) => draw_square(image, *x * w, *y * h, w, h, col),
            _ => (),
        }

        let col = fruit_color();

        match (
            self.fruit_x.try_into() as Result<usize, _>,
            self.fruit_y.try_into() as Result<usize, _>
        ) {
            (Ok(fx), Ok(fy)) => draw_square(image, fx * w, fy * h, w, h, col),
            _ => (),
        }
    }

    fn _point_in_snake(&self, x: usize, y: usize) -> bool {
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

    fn _point_in_fruit(&self, x: usize, y: usize) -> bool {
        match (
            self.fruit_x.try_into() as Result<usize, _>,
            self.fruit_y.try_into() as Result<usize, _>
        ) {
            (Ok(fx), Ok(fy)) => {
                let sx = fx * GameState::CELLSIZE;
                let sy = fy * GameState::CELLSIZE;
                let ex = sx + GameState::CELLSIZE;
                let ey = sy + GameState::CELLSIZE;

                let inx = (sx..=ex).contains(&x);
                let iny = (sy..=ey).contains(&y);

                inx && iny
            },
            _ => false
        }
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
                        if input.state == ElementState::Released {
                            return true
                        }
                        match input.virtual_keycode {
                            Some(VirtualKeyCode::Up) |
                            Some(VirtualKeyCode::W) => state.move_queue.push(( 0,  1)),
                            Some(VirtualKeyCode::Down) |
                            Some(VirtualKeyCode::S) => state.move_queue.push(( 0, -1)),
                            Some(VirtualKeyCode::Left) |
                            Some(VirtualKeyCode::A) => state.move_queue.push((-1,  0)),
                            Some(VirtualKeyCode::Right) |
                            Some(VirtualKeyCode::D) => state.move_queue.push(( 1,  0)),
                            Some(VirtualKeyCode::R) => {
                                if state.gameover {
                                    *state = GameState::new();
                                }
                            }
                            _ => (),
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

fn draw_square(
    image: &mut Image,
    x: usize, y: usize,
    w: usize, h: usize,
    col: Color
) {
    let width = image.width();

    for j in y..(y + h) {
        for i in x..(x + w) {
            let index = i + j * width;
            match image.get_mut(index) {
                Some(pixel) => {
                    *pixel = col
                },
                None => (),
            }
        }
    }
}
