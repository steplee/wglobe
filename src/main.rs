use std::{iter, sync::Arc};
use log::info;

use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

pub mod core;
pub mod renderables;

use core::{AppObjects, RenderState, BaseApp, UserApp, Renderable};
use renderables::{SimpleShape};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;


#[derive(Default)]
struct MyApp {
    renderables: Vec<Box<dyn Renderable>>,
}
impl UserApp for MyApp {
    fn render(&mut self, ao: &AppObjects) -> Result<(), wgpu::SurfaceError> {
        ao.window.request_redraw();

        // We can't render unless the surface is configured
        if !ao.is_surface_configured {
            return Ok(());
        }

        if self.renderables.len() == 0 {
            self.renderables.push(Box::new(crate::renderables::SimpleShape::new(ao)));
        }

        let output = ao.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let encoder = ao
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut rs = RenderState {
            ao: &ao,
            encoder: encoder,
            surface_tex_view: Some(view),
        };

        {
            for shape in &self.renderables {
                shape.render(&mut rs);
            }
        }

        ao.queue.submit(iter::once(rs.encoder.finish()));
        output.present();

        Ok(())
    }
    fn handle_key(&mut self, ao: &AppObjects, event_loop: &ActiveEventLoop, key: KeyCode, pressed: bool) {
        for r in &mut self.renderables {
            r.handle_key(event_loop, key, pressed);
        }
    }
    fn handle_mouse(&mut self, ao: &AppObjects, event_loop: &ActiveEventLoop, state: ElementState, button: MouseButton) {
    }
}


pub fn run() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // env_logger::init();
        let mut builder = env_logger::Builder::new();
        builder.filter_level(log::LevelFilter::Trace);
        builder.parse_env(env_logger::Env::new());
        builder.init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = BaseApp::<MyApp>::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );
    event_loop.run_app(&mut app)?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();

    Ok(())
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    run_web().unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    {
        run().unwrap();
    }
}
