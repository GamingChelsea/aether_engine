use anyhow::Context;
use bevy_ecs::prelude::*;
use engine_core::{AetherCore, TestBundle, Transform, Velocity};
use engine_i18n::t;
use engine_renderer::renderer::{InstanceData, Renderer};
use glam::Vec3;
use native_dialog::{DialogBuilder, MessageLevel};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop, window::Window,
};

struct App {
    renderer: Option<Renderer>,
    aether_core: AetherCore,
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
            self.aether_core.tick();
            self.renderer
                .as_mut()
                .expect(&t!("error.redraw_request"))
                .render()
                .expect(&t!("error.render"));
            self.about_to_wait(event_loop);
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
            Some(loc) => format!(
                "{}: {}, {}: {}",
                t!("error.file"),
                loc.file(),
                t!("error.line"),
                loc.line()
            ),
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

    let mut aether_core = AetherCore::new();
    let test_b = TestBundle {
        transform: Transform::default(),
        velocity: Velocity::default(),
    };
    aether_core.world.spawn(test_b);

    let mut app: App = App {
        renderer: None,
        aether_core: aether_core,
    };
    let event_loop = EventLoop::new().context(t!("error.event_loop_creation"))?;
    let renderer_a = app.renderer.take().expect("Renderer missing");

    app.aether_core.world.insert_resource(renderer_a);

    event_loop.run_app(&mut app).ok();
    Ok(())
}

pub fn sync_transfroms_system(query: Query<&Transform>, mut renderer: ResMut<Renderer>) {
    let mut instaces: Vec<InstanceData> = Vec::new();
    for a in query {
        instaces.push(InstanceData {
            position: Vec3 {
                x: a.x,
                y: a.y,
                z: a.z,
            },
        });
    }
    renderer.update_instances(&instaces);
}
