use sursface::app::AppState;
use sursface::display::Display;
use sursface::std::{clear, create_render_pipeline, create_shader, get_framebuffer};
use sursface::wgpu::{self, Color, RenderPipeline};

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        sursface::start::create_window_desktop::<TriangleState>(1280, 720);
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start_browser(canvas: sursface::wgpu::web_sys::HtmlCanvasElement) {
    sursface::start::create_window_browser::<TriangleState>(canvas);
}

struct TriangleState {
    render_pipeline: RenderPipeline,
}

impl AppState for TriangleState {
    fn new(display: &mut Display) -> Self {
        let device = &display.device;

        let shader = create_shader(device, include_str!("assets/shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = create_render_pipeline(display, pipeline_layout, shader, &[]);
        Self { render_pipeline }
    }

    fn draw(&mut self, display: &mut Display) {
        let clear_color = Color {
            r: 100.0 / 255.0,
            g: 149.0 / 255.0,
            b: 237.0 / 255.0,
            a: 255.0 / 255.0,
        };

        let device = &display.device;

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let (output, view) = get_framebuffer(&display.surface);
        {
            let mut rpass = clear(&view, &mut encoder, clear_color);

            rpass.set_pipeline(&self.render_pipeline);
            rpass.draw(0..3, 0..1);
        }

        display.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
