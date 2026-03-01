use anyhow::Context;
use engine_i18n::t;
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
        let window_attributes = Window::default_attributes().with_title(t!("ui.window_title"));
        let window = event_loop
            .create_window(window_attributes)
            .expect(&t!("error.window_creation"));

        let renderer = pollster::block_on(Renderer::new(Arc::new(window)))
            .expect(&t!("error.renderer_creation"));
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
                .expect(&t!("error.redraw_request"))
                .render()
                .expect(&t!("error.render"));
        }
        if let WindowEvent::Resized(new_size) = event {
            self.renderer
                .as_mut()
                .expect(&t!("error.resize"))
                .resize(new_size);
        }
    }
}

fn main() -> anyhow::Result<()> {
    engine_i18n::load("locales/de.toml");

    std::panic::set_hook(Box::new(|panic_info| {
        let message_buf: String;
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.as_str()
        } else {
            message_buf = t!("error.unknown");
            &message_buf
        };

        let location = match panic_info.location() {
            Some(loc) => format!("{}: {}, {}: {}", t!("error.file"), loc.file(), t!("error.line"), loc.line()),
            None => t!("error.unknown_location"),
        };
        DialogBuilder::message()
            .set_level(MessageLevel::Error)
            .set_title(&t!("ui.error_title"))
            .set_text(format!("{}\n\n{}", message, location))
            .alert()
            .show()
            .ok();
    }));

    let mut app: App = App { renderer: None };
    let event_loop = EventLoop::new().context(t!("error.event_loop_creation"))?;
    event_loop.run_app(&mut app).ok();
    Ok(())
}
