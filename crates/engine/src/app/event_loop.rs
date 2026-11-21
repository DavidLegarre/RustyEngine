use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::render::camera::Camera;
use crate::render::camera::CameraController;
use crate::render::renderer::Renderer;

pub struct App {
    pub window: Option<Arc<Window>>,
    pub renderer: Option<Renderer>,
    pub camera: Camera,
    pub camera_controller: CameraController,
    needs_redraw: bool,
    last_frame: Option<Instant>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            renderer: None,
            camera: Camera::default(),
            camera_controller: CameraController::default(),
            // Draw once on startup.
            needs_redraw: true,
            last_frame: None,
        }
    }
}

impl App {
    fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
        let PhysicalKey::Code(code) = event.physical_key else {
            return;
        };
        let pressed = event.state.is_pressed();

        if pressed && code == KeyCode::Escape {
            event_loop.exit();
            return;
        }

        if self.camera_controller.handle_key(code, pressed) {
            self.needs_redraw = true;
        }
    }

    fn init_renderer(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .expect("Failed to create window"),
        );

        match Renderer::new(window.clone()) {
            Ok(renderer) => {
                self.renderer = Some(renderer);
            }
            Err(e) => {
                eprintln!("Failed to initialize renderer: {e}");
                eprintln!("The window will open but no graphics will be drawn.");
            }
        }

        self.window = Some(window);
        self.needs_redraw = true;
    }

    fn draw(&mut self) {
        let delta = self
            .last_frame
            .map_or(0.0, |last| last.elapsed().as_secs_f32())
            // Clamp so a long idle pause does not teleport the camera.
            .min(0.1);
        self.last_frame = Some(Instant::now());

        self.camera_controller
            .update_camera(&mut self.camera, delta);

        let Self {
            renderer,
            camera,
            needs_redraw,
            camera_controller,
            ..
        } = self;
        if let Some(renderer) = renderer.as_mut() {
            renderer.render(camera);
        }
        // Keep animating only while a movement key is held.
        *needs_redraw = camera_controller.is_moving();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init_renderer(event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => self.handle_key(event_loop, &event),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size.width, size.height);
                }
                self.needs_redraw = true;
            }
            WindowEvent::RedrawRequested => self.draw(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if !self.needs_redraw {
            return;
        }
        if let Some(w) = &self.window {
            w.request_redraw();
        }
        self.needs_redraw = false;
    }
}
