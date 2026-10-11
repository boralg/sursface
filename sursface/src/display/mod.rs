use std::sync::Arc;

use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowAttributesExtWebSys;

pub struct Display<'a> {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'a>,
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

#[cfg(target_arch = "wasm32")]
pub(crate) struct DisplayWasmPending<'a> {
    canvas: wgpu::web_sys::HtmlCanvasElement,
    instance: wgpu::Instance,
    surface: wgpu::Surface<'a>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

pub struct DisplaySuspended {
    window: Arc<Window>,
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

fn instance() -> wgpu::Instance {
    wgpu::Instance::new(
        &wgpu::InstanceDescriptor {
            backends: if cfg!(target_arch = "wasm32") {
                wgpu::Backends::GL
            } else {
                wgpu::Backends::PRIMARY
            },
            ..Default::default()
        }
        .with_env(),
    )
}

async fn request_adapter_and_device(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'_>,
) -> (wgpu::Adapter, wgpu::Device, wgpu::Queue) {
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
}

fn surface_config(
    size: PhysicalSize<u32>,
    surface: &wgpu::Surface<'_>,
    adapter: &wgpu::Adapter,
) -> wgpu::SurfaceConfiguration {
    let surface_caps = surface.get_capabilities(adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);

    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        desired_maximum_frame_latency: 2,
        view_formats: vec![],
    }
}

#[cfg(target_arch = "wasm32")]
impl<'a> DisplayWasmPending<'a> {
    pub async fn from_canvas(canvas: wgpu::web_sys::HtmlCanvasElement) -> Self {
        let instance = instance();
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
            .expect("Couldn't create surface from canvas");

        let (adapter, device, queue) = request_adapter_and_device(&instance, &surface).await;
        Self {
            canvas,
            instance,
            surface,
            adapter,
            device,
            queue,
        }
    }

    pub fn into_display(self, event_loop: &ActiveEventLoop) -> Display<'a> {
        let window = Display::window_from_canvas(event_loop, self.canvas);
        let window = Arc::new(window);
        Display::new(
            window,
            self.instance,
            self.surface,
            self.adapter,
            self.device,
            self.queue,
        )
    }
}

impl<'a> Display<'a> {
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn from_window(window: Arc<Window>) -> Display<'static> {
        let instance = instance();
        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("Couldn't create surface");

        let (adapter, device, queue) = request_adapter_and_device(&instance, &surface).await;

        Display::new(window, instance, surface, adapter, device, queue)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn window_from_size(
        event_loop: &ActiveEventLoop,
        window_size: PhysicalSize<u32>,
    ) -> Window {
        event_loop
            .create_window(WindowAttributes::default().with_inner_size(window_size))
            .expect("Couldn't create window")
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn window_from_canvas(
        event_loop: &ActiveEventLoop,
        canvas: wgpu::web_sys::HtmlCanvasElement,
    ) -> Window {
        event_loop
            .create_window(WindowAttributes::default().with_canvas(Some(canvas)))
            .expect("Couldn't create window")
    }

    fn new(
        window: Arc<Window>,
        instance: wgpu::Instance,
        surface: wgpu::Surface<'a>,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
    ) -> Self {
        let config = surface_config(Self::initial_size(&window), &surface, &adapter);

        Self {
            window,
            instance,
            surface,
            adapter,
            device,
            queue,
            config,
        }
    }

    fn initial_size(window: &Window) -> PhysicalSize<u32> {
        let size = window.inner_size();

        #[cfg(target_arch = "wasm32")]
        if size.width == 0 || size.height == 0 {
            use winit::platform::web::WindowExtWebSys;

            if let (Some(canvas), Some(page)) = (window.canvas(), web_sys::window()) {
                let ratio = page.device_pixel_ratio();

                return PhysicalSize::new(
                    (canvas.client_width() as f64 * ratio).round() as u32,
                    (canvas.client_height() as f64 * ratio).round() as u32,
                );
            }
        }

        size
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        PhysicalSize::new(self.config.width, self.config.height)
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn suspend(self) -> DisplaySuspended {
        let Self {
            window,
            instance,
            adapter,
            device,
            queue,
            config,
            ..
        } = self;
        DisplaySuspended {
            window,
            instance,
            adapter,
            device,
            queue,
            config,
        }
    }
}

impl DisplaySuspended {
    pub fn resume(self) -> Display<'static> {
        let Self {
            window,
            instance,
            adapter,
            device,
            queue,
            config,
        } = self;

        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("Couldn't recreate surface");
        surface.configure(&device, &config);

        Display {
            window,
            instance,
            surface,
            adapter,
            device,
            queue,
            config,
        }
    }
}
