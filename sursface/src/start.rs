use winit::event_loop::{ControlFlow, EventLoop};

#[cfg(target_arch = "wasm32")]
use wgpu::web_sys::HtmlCanvasElement;

use crate::app::{App, AppState};

#[cfg(target_arch = "wasm32")]
extern crate console_error_panic_hook;

#[cfg(not(target_arch = "wasm32"))]
pub fn create_window_desktop<State: AppState + 'static>(
    width: u32,
    height: u32,
    init_state: State::InitState,
) {
    super::logger::init_logger();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::<State>::new_desktop(width, height, init_state);
    event_loop.run_app(&mut app).unwrap();
}

#[cfg(target_arch = "wasm32")]
pub fn create_window_browser<State: AppState + 'static>(
    canvas: HtmlCanvasElement,
    init_state: State::InitState,
) {
    super::logger::init_logger();
    console_error_panic_hook::set_once();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    spawn_local(async move {
        let mut app = App::<State>::new_wasm(canvas, init_state).await;
        event_loop.run_app(&mut app).unwrap();
    });
}

// TODO: android
