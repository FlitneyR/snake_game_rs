use std::{time::Instant, process::exit};
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
    let length = CELL_COUNT * CELL_SIZE;

    let canvas = Canvas::new(length, length)
        .state(GameState::new())
        .title("Snake\tWASD to move\tQ to quit\t\tR to reset")
        .input(GameState::input_handler);

    canvas.render(|state, image| {
        state.draw(image);

        state.framecount += 1;
    });
}

fn background() -> Color {
    Color {
        r: 25,
        g: 25,
        b: 25,
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
    should_erase_last: bool,
}

const CELL_SIZE: usize = 25;
const CELL_COUNT: usize = 25;
const SNAKE_BORDER: usize = 4;
const FRUIT_BORDER: usize = 2;

impl GameState {

    fn new() -> Self {
        Self {
            gameover: false,
            framecount: 0,
            head_x: 5,
            head_y: 5,
            head_dx: 1,
            head_dy: 0,
            to_grow: 3,
            body: vec![(4, 5), (5, 5)],
            fruit_x: -1,
            fruit_y: -1,
            last_tick: Instant::now(),
            move_queue: vec![],
            should_clear: true,
            should_erase_last: false,
        }
    }

    fn draw(&mut self, image: &mut Image) {
        if !self.should_draw_frame() || self.gameover {
            return;
        }

        if self.should_clear {
            draw_rect(image,
                0, 0,
                image.width(), image.height(),
                background()
            );

            self.should_clear = false;
        }

        if self.should_erase_last {
            self.erase_last(image);
        }

        self.update();

        self.draw_fruit(image);

        self.draw_head(image);

        self.extend_neck(image);

        self.last_tick = Instant::now();
    }

    fn update(&mut self) {
        self.perform_next_move();

        self.move_head();

        self.fruit_update();

        if self.self_collision() {
            self.gameover = true;
        }

        self.update_body_segments();
    }

    fn update_body_segments(&mut self) {
        self.body.push((self.head_x, self.head_y));
        self.to_grow -= 1;

        if self.should_erase_last {
            self.body.remove(0);
            self.should_erase_last = false;
        }

        if self.to_grow < 0 {
            self.to_grow += 1;
            self.should_erase_last = true;
        }
    }

    fn self_collision(&self) -> bool {
        self.body.iter().map(|(x, y)| {
            *x == self.head_x &&
            *y == self.head_y
        }).fold(false, |a, b| a || b)
    }

    fn perform_next_move(&mut self) {
        match self.move_queue.get(0) {
            Some((dx, dy)) => {
                if self.head_dx.abs() != dx.abs() &&
                   self.head_dy.abs() != dy.abs()
                {
                    self.head_dx = *dx;
                    self.head_dy = *dy;
                }
                self.move_queue.remove(0);
            },
            _ => ()
        }
    }

    fn move_head(&mut self) {
        (self.head_x, self.head_y) = wrap_add_2d(
            (self.head_x, self.head_y),
            (self.head_dx, self.head_dy),
             (CELL_COUNT, CELL_COUNT)
        );
    }

    fn fruit_update(&mut self) {
        if self.should_make_fruit() {
            let mut rng = rand::thread_rng();

            self.fruit_x = (rng.gen::<usize>() % CELL_COUNT) as isize;
            self.fruit_y = (rng.gen::<usize>() % CELL_COUNT) as isize;
        }

        if self.fruit_collision() {
            self.fruit_x = -1;
            self.fruit_y = -1;
            self.to_grow += 1;
        }
    }

    fn should_make_fruit(&self) -> bool {
        self.fruit_x < 0 &&
        self.fruit_y < 0
    }

    fn fruit_collision(&self) -> bool {
        match (
            self.fruit_x.try_into() as Result<usize, _>,
            self.fruit_y.try_into() as Result<usize, _>
        ) {
            (Ok(fx), Ok(fy)) => {
                self.body.iter().map(|(x, y)| {
                    fx == *x && fy == *y
                }).fold(false, |a, b| a || b)
            },
            _ => false,
        }
    }

    fn _erase(&self, image: &mut Image) {
        let w = CELL_SIZE;
        let h = w;
        let col = background();

        for (px, py) in self.body.iter() {
            draw_rect(image, *px * w, *py * h, w, h, col);
        }
    }

    fn should_draw_frame(&self) -> bool {
        self.last_tick.elapsed().as_millis() > 100
    }

    fn erase_last(&self, image: &mut Image) {
        let w = CELL_SIZE;
        let h = w;
        let col = background();

        match self.body.get(0) {
            Some((x, y)) => draw_rect(image, *x * w, *y * h, w, h, col),
            _ => ()
        }
    }

    fn draw_fruit(&self, image: &mut Image) {
        let w = CELL_SIZE;
        let h = w;
        let col = fruit_color();

        match (
            self.fruit_x.try_into() as Result<usize, _>,
            self.fruit_y.try_into() as Result<usize, _>
        ) {
            (Ok(fx), Ok(fy)) => draw_rect(image,
                fx * w + FRUIT_BORDER, fy * h + FRUIT_BORDER,
                w - FRUIT_BORDER * 2, h - FRUIT_BORDER * 2,
                col),
            _ => (),
        }
    }

    fn draw_head(&self, image: &mut Image) {
        let w = CELL_SIZE;
        let h = w;
        let col = background();

        let (x, y) = self.get_head();
        let (x, y) = (x * w, y * h);

        draw_rect(image, x, y, w, h, col);

        let col = snake_color();

        let (w, h) = (w - SNAKE_BORDER * 2, h - SNAKE_BORDER * 2);

        draw_rect(image, x + SNAKE_BORDER, y + SNAKE_BORDER, w, h, col);

        let (x, y) = match (
            self.head_dx, self.head_dy
        ) {
            (-1,  0) => (x + SNAKE_BORDER * 2, y + SNAKE_BORDER * 1),
            ( 1,  0) => (x + SNAKE_BORDER * 0, y + SNAKE_BORDER * 1),
            ( 0, -1) => (x + SNAKE_BORDER * 1, y + SNAKE_BORDER * 2),
            ( 0,  1) => (x + SNAKE_BORDER * 1, y + SNAKE_BORDER * 0),
            _ => (x, y),
        };

        draw_rect(image, x, y, w, h, col);
    }
    
    fn extend_neck(&self, image: &mut Image) {
        let (x, y) = self.get_head();

        let (x, y) = wrap_add_2d(
            (x, y),
            (-self.head_dx, -self.head_dy),
             (CELL_COUNT, CELL_COUNT)
        );

        let w = CELL_SIZE;
        let h = w;
        
        let (x, y) = (x * w, y * h);

        let (x, y) = match (
            self.head_dx, self.head_dy
        ) {
            ( 1,  0) => (x + SNAKE_BORDER * 2, y + SNAKE_BORDER * 1),
            (-1,  0) => (x + SNAKE_BORDER * 0, y + SNAKE_BORDER * 1),
            ( 0,  1) => (x + SNAKE_BORDER * 1, y + SNAKE_BORDER * 2),
            ( 0, -1) => (x + SNAKE_BORDER * 1, y + SNAKE_BORDER * 0),
            _ => (x, y),
        };

        let col = snake_color();

        draw_rect(image, x, y, w - SNAKE_BORDER * 2, h - SNAKE_BORDER * 2, col);
    }

    fn get_head(&self) -> (usize, usize) {
        (self.head_x, self.head_y)
    }

    fn _point_in_snake(&self, x: usize, y: usize) -> bool {
        for (px, py) in self.body.iter() {
            let sx = px * CELL_SIZE;
            let sy = py * CELL_SIZE;

            let ex = sx + CELL_SIZE;
            let ey = sy + CELL_SIZE;

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
                let sx = fx * CELL_SIZE;
                let sy = fy * CELL_SIZE;
                let ex = sx + CELL_SIZE;
                let ey = sy + CELL_SIZE;

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
                            Some(VirtualKeyCode::Q) => exit(0),
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

fn draw_rect(
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

fn wrap_add_2d((x, y): (usize, usize), (dx, dy): (isize, isize), (lx, ly): (usize, usize)) -> (usize, usize) {
    (wrap_add(x, dx, lx),
     wrap_add(y, dy, ly))
}

fn wrap_add(a: usize, b: isize, c: usize) -> usize {
    ((a as isize + b + c as isize) % c as isize) as usize
}
