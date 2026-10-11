use std::sync::Arc;
use winit::application::ApplicationHandler;
#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

#[cfg(target_arch = "wasm32")]
use super::display::DisplayWasmPending;
use super::display::{Display, DisplaySuspended};

pub trait AppState: Sized {
    type InitState;

    fn new(display: &mut Display, init_state: Self::InitState) -> Self;

    fn draw(&mut self, display: &mut Display);

    fn event(&mut self, display: &mut Display, event: WindowEvent) {
        let _ = (event, display);
    }

    fn device_event(&mut self, display: &mut Display, event: DeviceEvent) {
        let _ = (event, display);
    }

    fn exit_requested(&self) -> bool {
        false
    }
}

enum AppPhase<'a, State: AppState> {
    #[cfg(not(target_arch = "wasm32"))]
    DesktopPending {
        initial_size: PhysicalSize<u32>,
        init_state: State::InitState,
    },
    #[cfg(target_arch = "wasm32")]
    WasmPending {
        display_pending: DisplayWasmPending<'a>,
        init_state: State::InitState,
    },
    Running {
        display: Display<'a>,
        state: State,
    },
    Suspended {
        display: DisplaySuspended,
        state: State,
    },
}

pub(crate) struct App<'a, State: AppState> {
    phase: Option<AppPhase<'a, State>>,
}

impl<'a, State: AppState> App<'a, State> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_desktop(width: u32, height: u32, init_state: State::InitState) -> Self {
        log::debug!("Creating desktop app");
        App {
            phase: Some(AppPhase::DesktopPending {
                initial_size: PhysicalSize::new(width, height),
                init_state,
            }),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn new_wasm(
        canvas: wgpu::web_sys::HtmlCanvasElement,
        init_state: State::InitState,
    ) -> Self {
        log::debug!("Creating wasm app");

        let pending = DisplayWasmPending::from_canvas(canvas).await;

        App {
            phase: Some(AppPhase::WasmPending {
                display_pending: pending,
                init_state,
            }),
        }
    }
}

impl<'a, State: AppState> ApplicationHandler for App<'a, State> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let phase = self.phase.take().expect("App phase missing");

        self.phase = Some(match phase {
            #[cfg(not(target_arch = "wasm32"))]
            AppPhase::DesktopPending {
                initial_size,
                init_state,
            } => {
                let window = Display::window_from_size(event_loop, initial_size);
                let window = Arc::new(window);

                let mut display = pollster::block_on(Display::from_window(window));
                let state = State::new(&mut display, init_state);

                AppPhase::Running { display, state }
            }

            #[cfg(target_arch = "wasm32")]
            AppPhase::WasmPending {
                display_pending: pending,
                init_state,
            } => {
                let mut display = pending.into_display(event_loop);
                let state = State::new(&mut display, init_state);

                AppPhase::Running { display, state }
            }

            running @ AppPhase::Running { .. } => running,

            AppPhase::Suspended { display, state } => {
                log::debug!("Resuming");
                let display = display.resume();
                AppPhase::Running { display, state }
            }
        });
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        let phase = self.phase.take().expect("App phase missing");

        self.phase = Some(match phase {
            AppPhase::Running { display, state } => {
                log::debug!("Suspending");
                let display = display.suspend();
                AppPhase::Suspended { display, state }
            }
            other => other, // TODO: passthrough on all other states or just Suspended?
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // TODO: returning on suspended may leave released keys stuck
        let Some(AppPhase::Running { display, state }) = &mut self.phase else {
            return;
        };

        state.event(display, event.clone());

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                log::debug!("Window resized: {:?}", physical_size);
                display.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                state.draw(display);

                if state.exit_requested() {
                    event_loop.exit();
                    return;
                }

                display.window.as_ref().request_redraw();
            }
            _ => (),
        };
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let Some(AppPhase::Running { display, state }) = &mut self.phase else {
            return;
        };

        state.device_event(display, event.clone());
    }
}
