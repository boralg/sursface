use std::sync::{Arc, Mutex};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

#[cfg(target_arch = "wasm32")]
extern crate console_error_panic_hook;

use super::display::Display;

pub(crate) struct App<'a, State: AppState> {
    pub display: Option<Arc<Mutex<Display<'a>>>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub initial_size: PhysicalSize<u32>,
    #[cfg(target_arch = "wasm32")]
    pub canvas: wgpu::web_sys::HtmlCanvasElement,
    pub state: Option<Arc<Mutex<State>>>,
}

pub trait AppState {
    fn new(display: &mut Display) -> Self;
    fn create_display(window: Window) -> Display<'static> {
        Display::from_window(window)
    }

    fn draw(&mut self, display: &mut Display);

    fn event(&mut self, display: &mut Display, event: WindowEvent) {
        let (_, _) = (event, display); // suppress warning
    }
    fn device_event(&mut self, display: &mut Display, event: DeviceEvent) {
        let (_, _) = (event, display); // suppress warning
    }
}

fn init_logger() {
    #[cfg(target_arch = "wasm32")]
    {
        let base_level = log::LevelFilter::Info;
        let wgpu_level = log::LevelFilter::Error;

        fern::Dispatch::new()
            .level(base_level)
            .level_for("wgpu_core", wgpu_level)
            .level_for("wgpu_hal", wgpu_level)
            .level_for("naga", wgpu_level)
            .chain(fern::Output::call(console_log::log))
            .apply()
            .unwrap();
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let base_level = log::LevelFilter::Info;
        let wgpu_level = log::LevelFilter::Error;

        // parse_default_env will read the RUST_LOG environment variable and apply it on top
        // of these default filters
        env_logger::builder()
            .filter_level(base_level)
            .filter_module("wgpu_core", wgpu_level)
            .filter_module("wgpu_hal", wgpu_level)
            .filter_module("naga", wgpu_level)
            .parse_default_env()
            .init();
    }
}

impl<'a, State: AppState> App<'a, State> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_window_size(width: u32, height: u32) -> Self {
        log::debug!("Setting window size");
        App {
            initial_size: winit::dpi::PhysicalSize::new(width, height),
            display: None,
            state: None,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn from_canvas(canvas: wgpu::web_sys::HtmlCanvasElement) -> Self {
        log::debug!("Setting canvas size");
        App {
            canvas,
            display: None,
            state: None,
        }
    }
}

impl<'a, State: AppState> ApplicationHandler for App<'a, State> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        init_logger();

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.display = Some(Arc::new(Mutex::new(State::create_display(
                Display::create_window_from_size(event_loop, self.initial_size),
            ))));
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.display = Some(Arc::new(Mutex::new(Display::from_window(
                Display::create_window_from_canvas(event_loop, self.canvas.clone()),
            ))));
        }

        let new_state = State::new(&mut self.display.clone().unwrap().lock().unwrap());
        self.state = Some(Arc::new(Mutex::new(new_state)));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let mut display = self.display.as_ref().clone().unwrap().lock().unwrap();
        let mut state = self.state.as_ref().clone().unwrap().lock().unwrap();

        state.event(&mut display, event.clone());

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                log::debug!("Window resized: {:?}", physical_size);
                display.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                state.draw(&mut display);
                display.window.as_ref().request_redraw();
            }
            _ => (),
        };
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let mut display = self.display.as_ref().clone().unwrap().lock().unwrap();
        let mut state = self.state.as_ref().clone().unwrap().lock().unwrap();

        state.device_event(&mut display, event.clone());
    }
}
