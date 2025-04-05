use cgmath::{perspective, Deg, Matrix4, Point3, SquareMatrix, Vector3};
use sursface::app::AppState;
use sursface::display::Display;
use sursface::std::models::{cube, quad_uvs, VertexPositionNormalUv};
use sursface::std::{
    clear, create_render_pipeline, create_sampler_entry, create_shader, create_texture,
    create_texture_layout_entry_from_image, create_uniforms, get_framebuffer,
};
use sursface::wgpu::util::{BufferInitDescriptor, DeviceExt};
use sursface::wgpu::{
    BindGroup, BindGroupEntry, BindingResource, Buffer, BufferAddress, BufferUsages, Color,
    CommandEncoderDescriptor, PipelineLayoutDescriptor, RenderPipeline, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexStepMode,
};
use sursface::winit::dpi::PhysicalPosition;
use sursface::winit::event::{Touch, WindowEvent};

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        sursface::start::create_window_desktop::<CubeState>(1280, 720);
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start_browser(canvas: sursface::wgpu::web_sys::HtmlCanvasElement) {
    sursface::start::create_window_browser::<CubeState>(canvas);
}

struct CubeState {
    render_pipeline: RenderPipeline,
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,
    vertex_buffer: Buffer,
    texture_bind_group: BindGroup,
    uniforms: Uniforms,
    yaw: f64,
    pitch: f64,
    pan_speed: f64,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    model_view_proj: [[f32; 4]; 4],
    camera_pan: [[f32; 4]; 4],
}

impl AppState for CubeState {
    fn new(display: &mut Display) -> CubeState {
        let device = &display.device;

        let shader = create_shader(device, include_str!("assets/shader.wgsl"));

        let (texture_bind_group_entry, texture_view) = create_texture_layout_entry_from_image(
            device,
            &display.queue,
            include_bytes!("assets/dice.png"),
            0,
        );
        let (sampler_entry, sampler) = create_sampler_entry(device, 1);

        let (texture_bind_group_layout, texture_bind_group) = create_texture(
            device,
            &[texture_bind_group_entry, sampler_entry],
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        );

        let (uniform_buffer, uniform_bind_group_layout, uniform_bind_group) = create_uniforms(
            device,
            Uniforms {
                model_view_proj: Matrix4::identity().into(),
                camera_pan: Matrix4::identity().into(),
            },
            0,
        );

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = create_render_pipeline(
            display,
            pipeline_layout,
            shader,
            &[VertexBufferLayout {
                array_stride: std::mem::size_of::<VertexPositionNormalUv>() as BufferAddress,
                step_mode: VertexStepMode::Vertex,
                attributes: &[
                    VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: VertexFormat::Float32x3,
                    },
                    VertexAttribute {
                        offset: 12,
                        shader_location: 1,
                        format: VertexFormat::Float32x3,
                    },
                    VertexAttribute {
                        offset: 24,
                        shader_location: 2,
                        format: VertexFormat::Float32x2,
                    },
                ],
            }],
        );

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&cube(&[
                quad_uvs((0.00, 1f32 / 3f32), (0.25 + 0.00, 2f32 / 3f32)), // 6
                quad_uvs((0.50, 1f32 / 3f32), (0.25 + 0.50, 2f32 / 3f32)), // 1
                quad_uvs((0.25, 1f32 / 3f32), (0.25 + 0.25, 2f32 / 3f32)), // 4
                quad_uvs((0.75, 1f32 / 3f32), (0.25 + 0.75, 2f32 / 3f32)), // 3
                quad_uvs((0.50, 0f32 / 3f32), (0.25 + 0.50, 1f32 / 3f32)), // 2
                quad_uvs((0.50, 2f32 / 3f32), (0.25 + 0.50, 3f32 / 3f32)), // 5
            ])),
            usage: BufferUsages::VERTEX,
        });

        CubeState {
            render_pipeline,
            uniform_buffer,
            uniform_bind_group,
            vertex_buffer,
            texture_bind_group,
            uniforms: Uniforms {
                model_view_proj: Matrix4::identity().into(),
                camera_pan: Matrix4::identity().into(),
            },
            yaw: 0f64,
            pitch: 0f64,
            pan_speed: 0.4f64,
        }
    }

    fn draw(&mut self, display: &mut Display) {
        let clear_color = Color {
            r: 252.0 / 255.0,
            g: 241.0 / 255.0,
            b: 139.0 / 255.0,
            a: 255.0 / 255.0,
        };

        let output = {
            let mut encoder = display
                .device
                .create_command_encoder(&CommandEncoderDescriptor { label: None });

            let (output, view) = get_framebuffer(&display.surface);
            {
                let mut rpass = clear(&view, &mut encoder, clear_color);

                let aspect_ratio = display.config.width as f32 / display.config.height as f32;

                let model = Matrix4::identity();

                let view = Matrix4::look_at_rh(
                    Point3::new(3.0, 3.0, 3.0),
                    Point3::new(0.0, 0.0, 0.0),
                    Vector3::unit_y(),
                );
                let proj = perspective(Deg(45.0), aspect_ratio, 0.1, 10.0);
                let mvp = proj * view * model;

                self.uniforms.model_view_proj = mvp.into();

                let queue = &display.queue;
                queue.write_buffer(
                    &self.uniform_buffer,
                    0,
                    bytemuck::cast_slice(&[self.uniforms]),
                );

                {
                    rpass.set_pipeline(&self.render_pipeline);
                    rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
                    rpass.set_bind_group(1, &self.texture_bind_group, &[]);
                    rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                    rpass.draw(0..36, 0..1);
                }
            }

            {
                display.queue.submit(std::iter::once(encoder.finish()));
            }

            output
        };

        output.present();
    }

    fn event<'a>(&mut self, _display: &mut Display, event: WindowEvent) {
        let movement = {
            match event {
                WindowEvent::Touch(Touch { location, .. }) => Some(location),
                WindowEvent::CursorMoved { position, .. } => Some(position),
                _ => None,
            }
        };

        if let Some(PhysicalPosition { x, y }) = movement {
            self.yaw = x;
            self.pitch = -y;

            self.uniforms.camera_pan = (Matrix4::from_angle_y(Deg(self.yaw * self.pan_speed))
                * Matrix4::from_angle_x(Deg(self.pitch * self.pan_speed)))
            .cast()
            .unwrap()
            .into();
        }
    }
}
