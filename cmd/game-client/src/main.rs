use std::sync::Arc;

use engine_renderer::renderer::{self, Renderer};
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, event_loop::EventLoop,
    window::Window,
};

struct App {
    renderer: Option<Renderer>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Test Window");
        let window = event_loop.create_window(window_attributes).unwrap();

        let renderer = pollster::block_on(Renderer::new(Arc::new(window)));
        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if event == WindowEvent::CloseRequested {
            event_loop.exit();
        };
        if event == WindowEvent::RedrawRequested {
            self.renderer.as_mut().unwrap().render();
        }
        if let WindowEvent::Resized(new_size) = event {
            self.renderer.as_mut().unwrap().resize(new_size);
        }
    }
}

fn main() {
    let mut app: App = App { renderer: None };
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut app).ok();
}
