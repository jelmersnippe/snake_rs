use std::sync::Arc;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes},
};

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Example");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());

        let pixels = Pixels::new(size.width, size.height, surface_texture).unwrap();

        self.pixels = Some(pixels);
        self.window = Some(window);

        // kick off first frame
        self.window.as_ref().unwrap().request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                println!("Close requested: stopping");
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                let pixels = self.pixels.as_mut().unwrap();
                let frame = pixels.frame_mut();

                // Fill screen with a color (RGBA)
                for pixel in frame.chunks_exact_mut(4) {
                    pixel[0] = 0x20; // R
                    pixel[1] = 0x80; // G
                    pixel[2] = 0xFF; // B
                    pixel[3] = 0xFF; // A
                }

                pixels.render().unwrap();

                // request next frame (simple game loop)
                self.window.as_ref().unwrap().request_redraw();
            }
            winit::event::WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
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
