use anyhow::Context;
use engine_renderer::renderer::Renderer;
use native_dialog::{DialogBuilder, MessageLevel};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop, window::Window,
};

struct App {
    renderer: Option<Renderer>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Test Window");
        let window = event_loop
            .create_window(window_attributes)
            .expect("Window Creation Error");

        let renderer = pollster::block_on(Renderer::new(Arc::new(window)))
            .expect("Render could not be created");
        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if event == WindowEvent::CloseRequested {
            event_loop.exit();
        };
        if event == WindowEvent::RedrawRequested {
            self.renderer
                .as_mut()
                .expect("Redraw Request Error")
                .render()
                .expect("Render Error");
        }
        if let WindowEvent::Resized(new_size) = event {
            self.renderer
                .as_mut()
                .expect("Rezise Error")
                .resize(new_size);
        }
    }
}

fn main() -> anyhow::Result<()> {
    std::panic::set_hook(Box::new(|panic_info| {
        let message = panic_info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .unwrap_or("Unkown Error");

        let location = match panic_info.location() {
            Some(loc) => format!("File: {}, Line: {}", loc.file(), loc.line()),
            None => String::from("Unkown location"),
        };
        DialogBuilder::message()
            .set_level(MessageLevel::Error)
            .set_title("Aether Engine — Error")
            .set_text(format!("{}\n\n{}", message, location))
            .alert()
            .show()
            .ok();
    }));

    let mut app: App = App { renderer: None };
    let event_loop = EventLoop::new().context("EventLoop could not be created")?;
    event_loop.run_app(&mut app).ok();
    Ok(())
}
