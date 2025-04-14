use sursface::app::AppState;
use sursface::display::Display;
use sursface::wgpu::{self, TextureView};

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        sursface::start::create_window_desktop::<EmptyState>(1280, 720);
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start_browser(canvas: sursface::wgpu::web_sys::HtmlCanvasElement) {
    sursface::start::create_window_browser::<EmptyState>(canvas);
}

#[derive(Clone)]
struct EmptyState {}

impl AppState for EmptyState {
    fn new<'a>(_display: &mut Display) -> Self {
        Self {}
    }

    fn draw<'a>(&mut self, display: &mut Display) {
        let output = display.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        clear(
            display,
            &view,
            wgpu::Color {
                r: 100.0 / 255.0,
                g: 149.0 / 255.0,
                b: 237.0 / 255.0,
                a: 1.0,
            },
        );

        output.present();
    }
}

fn clear<'a>(display: &mut Display, view: &TextureView, color: sursface::wgpu::Color) {
    let mut encoder = display
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });

    {
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
    }

    display.queue.submit(std::iter::once(encoder.finish()));
}
