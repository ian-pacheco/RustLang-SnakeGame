//! Snake game
//!
//! Example of 2d graphics in Rust.
//! Firsts steps with RustLang
//! Completed code for https://youtu.be/HCwMb0KslX8
//! Author: ian-pacheco

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};

use std::collections::LinkedList;
use std::iter::FromIterator;

#[derive(Clone, PartialEq)]
enum Direction {
    Right,
    Left,
    Up,
    Down,
}

pub struct Food {
    x: u32,
    y: u32,
}

impl Food {
    fn update(&mut self, s: &Snake) -> bool {
        let front = s.snake_parts.front().unwrap();
        if front.0 == self.x && front.1 == self.y {
            true
        } else {
            false
        }
    }

    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs, width: u32) {
        let black: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        let x = self.x * width;
        let y = self.y * width;
        let square = graphics::rectangle::square(x as f64, y as f64, width as f64);

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            graphics::rectangle(black, square, transform, gl)
        });
    }
}

pub struct Game {
    gl: GlGraphics,
    snake: Snake,
    food: Food,
    just_eaten: bool,
    cols: u32,
    rows: u32,
    score: u32,
    square_width: u32    
}

impl Game {
    fn render(&mut self, arg: &RenderArgs) {

        let green: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

        self.gl.draw(arg.viewport(), |_c, gl| {
            graphics::clear(green, gl);
        });

        self.snake.render(arg);
        self.food.render(&mut self.gl, arg, self.square_width);
    }

    fn update(&mut self, arg: &UpdateArgs) -> bool {
        if !self.snake.update(self.just_eaten, self.cols, self.rows) {
            return false;
        }

        if self.just_eaten {
            self.score += 1;
            self.just_eaten = false;
        }

        self.just_eaten = self.food.update(&self.snake);
        if self.just_eaten {
            use rand::Rng;
            use rand::thread_rng;

            let mut r = thread_rng();
            loop {
                let new_x = r.gen_range(0, self.cols);
                let new_y = r.gen_range(0, self.rows);
                if !self.snake.is_collide(new_x, new_y) {
                    self.food = Food { x: new_x, y: new_y };
                    break;
                }
            }
        }
        true
    }

    fn pressed(&mut self, btn: &Button) {
        let last_direction = self.snake.dir.clone();

        self.snake.dir = match btn {
            &Button::Keyboard(Key::Up) if last_direction != Direction::Down => Direction::Up,
            &Button::Keyboard(Key::Down) if last_direction != Direction::Up => Direction::Down,
            &Button::Keyboard(Key::Left) if last_direction != Direction::Right => Direction::Left,
            &Button::Keyboard(Key::Right) if last_direction != Direction::Left => Direction::Right,
            _ => last_direction,
        };
    }
}

struct Snake {
    dir: Direction,
    gl: GlGraphics,
    snake_parts: LinkedList<(Snake_Piece)>,
    width: u32
}

#[derive(Clone)]
pub struct Snake_Piece(u32, u32);

impl Snake {
    fn render(&mut self, args: &RenderArgs) {

        let red: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let squares: Vec<graphics::types::Rectangle> = self.snake_parts
            .iter()
            .map(|p| Snake_Piece(p.0 * self.width, p.1 * self.width))
            .map(|p| graphics::rectangle::square(p.0 as f64, p.1 as f64, self.width as f64))
            .collect();        

        self.gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            squares.into_iter()
                   .for_each(|square| graphics::rectangle(red, square, transform, gl));
        });
    }

    fn update(&mut self, just_eaten: bool, cols: u32, rows: u32) -> bool {
        let mut new_front: Snake_Piece = 
        (*self.snake_parts.front().expect("No front of snake found.")).clone();

        if (self.dir == Direction::Up && new_front.1 == 0)
            || (self.dir == Direction::Left && new_front.0 == 0)
            || (self.dir == Direction::Down && new_front.1 == rows - 1)
            || (self.dir == Direction::Right && new_front.0 == cols - 1) {
                return false;
            }

        match self.dir {
            Direction::Up => new_front.1 -= 1,
            Direction::Down => new_front.1 += 1,
            Direction::Left => new_front.0 -= 1,
            Direction::Right => new_front.0 += 1
        }

        if !just_eaten {
            self.snake_parts.pop_back();
        }

        if self.is_collide(new_front.0, new_front.1) {
            return false;
        }
        self.snake_parts.push_front(new_front);
        true
    }

    fn is_collide(&self, x: u32, y: u32) -> bool {
        self.snake_parts.iter().any(|p| x == p.0 && y == p.1)
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    let cols: u32 = 30;
    let rows: u32 = 20;
    let square_width: u32 = 20;

    let width = cols * square_width;
    let height = rows * square_width;

    let mut window: GlutinWindow = WindowSettings::new("Snake Game", [width, height])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut game = Game {
        gl: GlGraphics::new(opengl),
        rows: rows,
        cols: cols,
        square_width: square_width,
        just_eaten: false,
        food: Food { x: 1, y: 1 },
        score: 0,
        snake: Snake {
            gl: GlGraphics::new(opengl),
            snake_parts: LinkedList::from_iter((vec![Snake_Piece(cols / 2, rows /2)]).into_iter()),
            width: square_width,
            dir: Direction::Down
        }
    };

    let mut events = Events::new(EventSettings::new()).ups(10);
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        if let Some(u) = e.update_args() {
            if !game.update(&u) {
                break;
            }
        }

        if let Some(k) = e.button_args() {
            if k.state == ButtonState::Press {
                game.pressed(&k.button);
            }
        }
    }
        println!("Congratulations, your score was: {}", game.score);
}
