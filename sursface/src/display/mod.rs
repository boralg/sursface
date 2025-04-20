use std::sync::Arc;

use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowAttributesExtWebSys;

pub struct Display<'a> {
    pub window: Arc<Window>,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

impl<'a> Display<'a> {
    pub(crate) fn create_window_from_size(
        event_loop: &ActiveEventLoop,
        window_size: PhysicalSize<u32>,
    ) -> Window {
        let window = event_loop
            .create_window(WindowAttributes::default().with_inner_size(window_size))
            .expect("Couldn't create window");

        window
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn create_window_from_canvas(
        event_loop: &ActiveEventLoop,
        canvas: wgpu::web_sys::HtmlCanvasElement,
    ) -> Window {
        let window = event_loop
            .create_window(WindowAttributes::default().with_canvas(Some(canvas)))
            .expect("Couldn't create window");

        window
    }

    pub fn from_window(window: Window) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let window = Arc::new(window);
        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("Couldn't create surface");

        let (adapter, device, queue) = pollster::block_on(async {
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .expect("Couldn't get adapter");

            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    memory_hints: wgpu::MemoryHints::Performance,
                    trace: wgpu::Trace::Off,
                })
                .await
                .unwrap();

            (adapter, device, queue)
        });

        let size = window.inner_size();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        Self {
            window,
            size,
            surface,
            device,
            queue,
            config,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}
