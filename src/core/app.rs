use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use super::AppObjects;
use super::Camera;

pub struct RenderState<'a> {
    pub ao: &'a AppObjects,

    pub encoder: wgpu::CommandEncoder,

    pub surface_tex_view: Option<wgpu::TextureView>,

    pub cam: &'a Camera,

    // model: [f32; 16],
    // view: [f32; 16],
    // proj: [f32; 16],
}

pub trait Renderable {
    fn render(self: &Self, rs: &mut RenderState);

    fn handle_key(&mut self, _event_loop: &ActiveEventLoop, _key: KeyCode, _pressed: bool) {}
    fn handle_mouse(&mut self, _event_loop: &ActiveEventLoop, _state: ElementState, _button: MouseButton) {}
}

pub trait UserApp: Default {
    fn render(&mut self, ao: &AppObjects) -> Result<(), wgpu::SurfaceError>;
    fn handle_key(&mut self, ao: &AppObjects, event_loop: &ActiveEventLoop, key: KeyCode, pressed: bool);
    fn handle_mouse(&mut self, ao: &AppObjects, event_loop: &ActiveEventLoop, state: ElementState, button: MouseButton);
}

pub struct BaseApp<UApp : UserApp> {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<AppObjects>>,
    pub ao: Option<AppObjects>,

    pub uapp: UApp,
}


impl<UApp: UserApp> BaseApp<UApp> {
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<AppObjects>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            ao: None,
            #[cfg(target_arch = "wasm32")]
            proxy,
            uapp: Default::default(),
        }
    }
}

impl<UApp: UserApp> ApplicationHandler<AppObjects> for BaseApp<UApp> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            // If we are not on web we can use pollster to
            // await the
            self.ao = Some(pollster::block_on(AppObjects::new(window)).unwrap());
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(proxy
                        .send_event(
                            AppObjects::new(window)
                                .await
                                .expect("Unable to create canvas!!!")
                        )
                        .is_ok())
                });
            }
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: AppObjects) {
        #[cfg(target_arch = "wasm32")]
        {
            event.window.request_redraw();
            event.resize(
                event.window.inner_size().width,
                event.window.inner_size().height,
            );
        }
        self.ao = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let ao = match &mut self.ao {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => ao.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                match self.uapp.render(ao) {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = ao.window.inner_size();
                        ao.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
            }
            WindowEvent::MouseInput { device_id: _device_id, state: mstate, button } => {
                self.uapp.handle_mouse(ao, event_loop, mstate, button);
            },
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                match (code, key_state.is_pressed()) {
                    (KeyCode::Escape, true) => event_loop.exit(),
                    _ => self.uapp.handle_key(ao, event_loop, code, key_state.is_pressed())
                }
            }
            _ => {}
        }
    }
}
