use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::render::renderer::Renderer;

pub struct App {
    pub window: Option<Arc<Window>>,
    pub renderer: Option<Renderer>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            renderer: None,
        }
    }
}

impl App {
    fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
        if event.state.is_pressed() && event.physical_key == PhysicalKey::Code(KeyCode::Escape) {
            event_loop.exit();
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
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = self.renderer.as_ref() {
                    renderer.render();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}
