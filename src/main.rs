use rand::RngExt;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use pixels::{Pixels, SurfaceTexture};
use wgpu::Color;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::Window,
};

enum Direction {
    Up,
    Right,
    Down,
    Left,
}

struct SnakePart {
    pub x: i16,
    pub y: i16,
}

struct Snake {
    pub parts: Vec<SnakePart>,
    pub direction: Direction,
}

impl Snake {
    pub fn new() -> Self {
        Self {
            direction: Direction::Up,
            parts: vec![
                SnakePart { x: 200, y: 200 },
                SnakePart { x: 200, y: 220 },
                SnakePart { x: 200, y: 240 },
            ],
        }
    }
}

struct GameState {
    snake: Snake,
    apple_pos_x: i16,
    apple_pos_y: i16,
}

impl GameState {
    pub fn move_apple(&mut self) {
        let mut rng = rand::rng();

        self.apple_pos_x = rng.random_range(0..20) * BLOCK_SIZE;
        self.apple_pos_y = rng.random_range(0..20) * BLOCK_SIZE;

        if is_in_snake(self.apple_pos_x, self.apple_pos_y, &self.snake) {
            self.move_apple();
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        let mut rng = rand::rng();
        Self {
            snake: Snake::new(),
            apple_pos_x: rng.random_range(0..20) * BLOCK_SIZE,
            apple_pos_y: rng.random_range(0..20) * BLOCK_SIZE,
        }
    }
}

const FPS: u64 = 2;

struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    state: GameState,
    next_frame_time: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: Default::default(),
            pixels: Default::default(),
            state: Default::default(),
            next_frame_time: Instant::now(),
        }
    }
}

impl App {
    fn update(&mut self) {
        let (velocity_x, velocity_y) = match self.state.snake.direction {
            Direction::Up => (0, -BLOCK_SIZE),
            Direction::Down => (0, BLOCK_SIZE),
            Direction::Right => (BLOCK_SIZE, 0),
            Direction::Left => (-BLOCK_SIZE, 0),
        };

        let mut prev_x = self.state.snake.parts[0].x;
        let mut prev_y = self.state.snake.parts[0].y;

        self.state.snake.parts[0].x += velocity_x;
        self.state.snake.parts[0].y += velocity_y;

        for i in 1..self.state.snake.parts.len() {
            let x = self.state.snake.parts[i].x;
            let y = self.state.snake.parts[i].y;

            self.state.snake.parts[i].x = prev_x;
            self.state.snake.parts[i].y = prev_y;

            prev_x = x;
            prev_y = y;
        }

        if is_in_snake(
            self.state.apple_pos_x,
            self.state.apple_pos_y,
            &self.state.snake,
        ) {
            self.state.snake.parts.push(SnakePart {
                x: prev_x,
                y: prev_y,
            });

            self.state.move_apple();
        }
    }

    fn draw(&mut self) {
        let pixels = self.pixels.as_mut().unwrap();
        let frame = pixels.frame_mut();

        // Fill screen with a color (RGBA)
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;

            if is_in_snake(x, y, &self.state.snake) {
                pixel.copy_from_slice(&SNAKE_COLOR);
            } else if is_in_area(x, y, self.state.apple_pos_x, self.state.apple_pos_y) {
                pixel.copy_from_slice(&APPLE_COLOR);
            } else {
                pixel.copy_from_slice(&BG_COLOR);
            }
        }
        pixels.render().unwrap();
    }
}

const WIDTH: i16 = 400;
const HEIGHT: i16 = 400;
const BLOCK_SIZE: i16 = 20;
const SNAKE_COLOR: [u8; 4] = [0x20, 0x80, 0x20, 0xFF];
const APPLE_COLOR: [u8; 4] = [0x80, 0x20, 0x20, 0xFF];
const BG_COLOR: [u8; 4] = [0x20, 0x20, 0x80, 0xFF];

fn is_in_snake(x: i16, y: i16, snake: &Snake) -> bool {
    for part in &snake.parts {
        if is_in_area(x, y, part.x, part.y) {
            return true;
        }
    }

    false
}

fn is_in_area(x: i16, y: i16, area_x: i16, area_y: i16) -> bool {
    x >= area_x && x < area_x + BLOCK_SIZE && y >= area_y && y < area_y + BLOCK_SIZE
}

impl ApplicationHandler for App {
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();

        if now >= self.next_frame_time {
            self.update();
            self.draw();

            self.next_frame_time += Duration::from_millis(1000 / FPS);
        }

        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame_time));
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Example")
            .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());

        let mut pixels = Pixels::new(size.width, size.height, surface_texture).unwrap();
        pixels.clear_color(Color::RED);

        self.pixels = Some(pixels);
        self.window = Some(window);

        self.state.move_apple();

        self.next_frame_time = Instant::now();

        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame_time));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::KeyboardInput { event, .. } if !event.repeat => {
                match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(key_code) => match key_code {
                        winit::keyboard::KeyCode::ArrowLeft => {
                            self.state.snake.direction = Direction::Left;
                        }
                        winit::keyboard::KeyCode::ArrowRight => {
                            self.state.snake.direction = Direction::Right;
                        }
                        winit::keyboard::KeyCode::ArrowUp => {
                            self.state.snake.direction = Direction::Up;
                        }
                        winit::keyboard::KeyCode::ArrowDown => {
                            self.state.snake.direction = Direction::Down;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    let mut app = App::default();

    event_loop.set_control_flow(ControlFlow::Wait);

    let result = event_loop.run_app(&mut app);
    match result {
        Ok(ok) => println!("Ok: {:?}", ok),
        Err(err) => println!("Error: {:?}", err),
    }
}
