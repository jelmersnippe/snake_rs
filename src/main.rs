use rand::RngExt;
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
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

struct GameState {
    snake_direction: Direction,
    snake_pos_x: i16,
    snake_pos_y: i16,
    snake_length: i16,
    apple_pos_x: i16,
    apple_pos_y: i16,
}

impl Default for GameState {
    fn default() -> Self {
        let mut rng = rand::rng();
        Self {
            snake_direction: Direction::Up,
            snake_pos_x: 200,
            snake_pos_y: 200,
            snake_length: 3,
            apple_pos_x: rng.random_range(0..20) * BLOCK_SIZE,
            apple_pos_y: rng.random_range(0..20) * BLOCK_SIZE,
        }
    }
}

const FPS: u128 = 1;

struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    state: GameState,
    last_update: SystemTime,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: Default::default(),
            pixels: Default::default(),
            state: Default::default(),
            last_update: SystemTime::now(),
        }
    }
}

impl App {
    fn update(&mut self) {
        let time_since_last_update = SystemTime::now()
            .duration_since(self.last_update)
            .expect("")
            .as_millis();

        if time_since_last_update < 1000 * FPS {
            return;
        }

        let (snake_x, snake_y) = match self.state.snake_direction {
            Direction::Up => (0, -BLOCK_SIZE),
            Direction::Down => (0, BLOCK_SIZE),
            Direction::Right => (BLOCK_SIZE, 0),
            Direction::Left => (-BLOCK_SIZE, 0),
        };
        self.state.snake_pos_x += snake_x;
        self.state.snake_pos_y += snake_y;

        self.last_update = SystemTime::now();
    }

    fn draw(&mut self) {
        let pixels = self.pixels.as_mut().unwrap();
        let frame = pixels.frame_mut();

        // Fill screen with a color (RGBA)
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / HEIGHT as usize) as i16;

            if is_snake(
                x,
                y,
                self.state.snake_pos_x,
                self.state.snake_pos_y,
                self.state.snake_length,
                &self.state.snake_direction,
            ) {
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

fn is_snake(
    x: i16,
    y: i16,
    snake_x: i16,
    snake_y: i16,
    snake_length: i16,
    snake_direction: &Direction,
) -> bool {
    for i in 0..snake_length {
        let (snake_x, snake_y) = match snake_direction {
            Direction::Up => (snake_x, snake_y - (BLOCK_SIZE * i)),
            Direction::Down => (snake_x, snake_y + (BLOCK_SIZE * i)),
            Direction::Right => (snake_x + (BLOCK_SIZE * i), snake_y),
            Direction::Left => (snake_x - (BLOCK_SIZE * i), snake_y),
        };
        if is_in_area(x, y, snake_x, snake_y) {
            return true;
        }
    }

    false
}

fn is_in_area(x: i16, y: i16, area_x: i16, area_y: i16) -> bool {
    x > area_x && x < area_x + BLOCK_SIZE && y > area_y && y < area_y + BLOCK_SIZE
}

impl ApplicationHandler for App {
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

        // kick off first frame
        self.window.as_ref().unwrap().request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                println!("Close requested: stopping");
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                self.draw();

                // request next frame (simple game loop)
                self.update();
                self.window.as_ref().unwrap().request_redraw();
            }
            winit::event::WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } if !event.repeat => {
                match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(key_code) => match key_code {
                        winit::keyboard::KeyCode::ArrowLeft => {
                            self.state.snake_direction = Direction::Left
                        }
                        winit::keyboard::KeyCode::ArrowRight => {
                            self.state.snake_direction = Direction::Right
                        }
                        winit::keyboard::KeyCode::ArrowUp => {
                            self.state.snake_direction = Direction::Up
                        }
                        winit::keyboard::KeyCode::ArrowDown => {
                            self.state.snake_direction = Direction::Down
                        }
                        _ => {}
                    },
                    _ => {}
                }
                println!("{:?} {:?} {:?}", device_id, event, is_synthetic)
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();

    let result = event_loop.run_app(&mut app);
    match result {
        Ok(ok) => println!("Ok: {:?}", ok),
        Err(err) => println!("Error: {:?}", err),
    }
}
