use bytemuck::{Pod, Zeroable};
use std::fmt::Display as FmtDisplay;
use sursface::app::AppState;
use sursface::cgmath::{Vector2, Zero};
use sursface::display::Display;
use sursface::std::models::{quad_no_normal, quad_uvs, VertexPositionUv};
use sursface::std::{
    clear, create_render_pipeline, create_shader, create_uniforms, get_framebuffer,
};
use sursface::time::now_secs;
use sursface::wgpu::util::DeviceExt;
use sursface::wgpu::{
    BindGroup, Buffer, BufferAddress, BufferUsages, Color, CommandEncoderDescriptor,
    PipelineLayoutDescriptor, RenderPipeline, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexStepMode,
};
use sursface::winit::dpi::PhysicalPosition;
use sursface::winit::event::{ElementState, MouseButton, Touch, TouchPhase, WindowEvent};
use sursface::{log, wgpu};

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        sursface::start::create_window_desktop::<MandelbrotState>(720, 720);
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start_browser(canvas: sursface::wgpu::web_sys::HtmlCanvasElement) {
    sursface::start::create_window_browser::<MandelbrotState>(canvas);
}

#[derive(Clone)]
enum InteractionState {
    /// Not pressing or holding down.
    Idle { pre_tapped_at: Option<f32> },
    /// Holding down but not dragging yet.
    PanningIdle { pressed_down_at: f32 },
    /// Holding down and dragging.
    Panning,
    /// Holding down without dragging for a bit then possibly dragging.
    ZoomingIn,
    /// Holding down without dragging for a bit (with pre-tap) then possibly dragging.
    ZoomingOut,
}

impl FmtDisplay for InteractionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InteractionState::Idle { pre_tapped_at } => {
                write!(f, "Idle {{ pre_tapped_at: {:?} }}", pre_tapped_at)
            }
            InteractionState::PanningIdle { pressed_down_at } => {
                write!(f, "PanningIdle {{ pressed_down_at: {} }}", pressed_down_at)
            }
            InteractionState::Panning => write!(f, "Panning"),
            InteractionState::ZoomingIn => write!(f, "Zooming"),
            InteractionState::ZoomingOut => write!(f, "ZoomingOut"),
        }
    }
}

struct MandelbrotState {
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,
    uniforms: Uniforms,
    scale_speed: f32,
    last_cursor_location: PhysicalPosition<f32>,
    cursor_location: PhysicalPosition<f32>,
    last_timestep: f32,
    interaction_state: InteractionState,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Uniforms {
    translation: [f32; 2], // 8 bytes
    cursor_pos: [f32; 2],  // 8 bytes
    scale: f32,            // 4 bytes
    aspect_ratio: f32,     // 4 bytes
    _padding: [f32; 2],    // 8 bytes to make the struct size 32 bytes
}

impl AppState for MandelbrotState {
    fn new(display: &mut Display) -> MandelbrotState {
        let device = &display.device;
        let aspect_ratio = display.config.width as f32 / display.config.height as f32;

        let shader = create_shader(device, include_str!("assets/shader.wgsl"));
        let (uniform_buffer, uniform_bind_group_layout, uniform_bind_group) = create_uniforms(
            device,
            Uniforms {
                translation: Vector2::zero().into(),
                cursor_pos: Vector2::zero().into(),
                scale: 4.0,
                aspect_ratio,
                _padding: [0.0; 2],
            },
            0,
        );

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = create_render_pipeline(
            display,
            pipeline_layout,
            shader,
            &[VertexBufferLayout {
                array_stride: std::mem::size_of::<VertexPositionUv>() as BufferAddress,
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
                        format: VertexFormat::Float32x2,
                    },
                ],
            }],
        );

        let quad_uvs = quad_uvs((0.0, 0.0), (1.0, 1.0));

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&quad_no_normal(
                [-1.0, -1.0, 1.0],
                [1.0, -1.0, 1.0],
                [1.0, 1.0, 1.0],
                [-1.0, 1.0, 1.0],
                quad_uvs,
            )),
            usage: BufferUsages::VERTEX,
        });

        MandelbrotState {
            render_pipeline,
            vertex_buffer,
            uniform_buffer,
            uniform_bind_group,
            uniforms: Uniforms {
                translation: Vector2::zero().into(),
                cursor_pos: Vector2::zero().into(),
                scale: 4.0,
                aspect_ratio,
                _padding: [0.0; 2],
            },
            scale_speed: 1.0 - 0.001,
            last_cursor_location: PhysicalPosition::new(0.0, 0.0),
            cursor_location: PhysicalPosition::new(0.0, 0.0),
            last_timestep: now_secs(),
            interaction_state: InteractionState::Idle {
                pre_tapped_at: None,
            },
        }
    }

    fn draw(&mut self, display: &mut Display) {
        let dt = {
            let mut dt = 0f32;
            dt += now_secs() - self.last_timestep;

            #[cfg(target_arch = "wasm32")]
            {
                dt *= -1000f32; // hack
            }

            dt
        };

        self.last_timestep = now_secs();
        self.uniforms.aspect_ratio = display.config.width as f32 / display.config.height as f32;

        let clear_color = Color {
            r: 100.0 / 255.0,
            g: 149.0 / 255.0,
            b: 237.0 / 255.0,
            a: 255.0 / 255.0,
        };

        self.interaction_state = match self.interaction_state.clone() {
            state @ InteractionState::PanningIdle { pressed_down_at } => {
                if now_secs() - pressed_down_at > 1f32 {
                    log::info!("Started zooming in at {}", now_secs());
                    InteractionState::ZoomingIn
                } else {
                    state
                }
            }
            state => state,
        };

        match self.interaction_state {
            InteractionState::ZoomingIn => {
                self.uniforms.scale *= self.scale_speed.powf(1f32 - dt as f32);
            }
            InteractionState::ZoomingOut => {
                self.uniforms.scale /= self.scale_speed.powf(1f32 - dt as f32);
            }
            _ => (),
        }

        let output = {
            let mut encoder = display
                .device
                .create_command_encoder(&CommandEncoderDescriptor { label: None });

            let (output, view) = get_framebuffer(&display.surface);

            {
                let mut rpass = clear(&view, &mut encoder, clear_color);

                self.uniforms.cursor_pos = [
                    self.cursor_location.x / display.config.width as f32,
                    self.cursor_location.y / display.config.height as f32,
                ];

                let queue = &display.queue;
                queue.write_buffer(
                    &self.uniform_buffer,
                    0,
                    bytemuck::cast_slice(&[self.uniforms]),
                );

                {
                    rpass.set_pipeline(&self.render_pipeline);
                    rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
                    rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                    rpass.draw(0..6, 0..1);
                }
            }

            display.queue.submit(std::iter::once(encoder.finish()));

            output
        };

        output.present();
    }

    fn event<'a>(&mut self, display: &mut Display, event: WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => self.handle_cursor_move(display, position),
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => self.interaction_state = self.handle_cursor_down(),
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => self.interaction_state = self.handle_cursor_up(),
            WindowEvent::Touch(Touch {
                phase: TouchPhase::Moved,
                location,
                ..
            }) => self.handle_cursor_move(display, location),
            WindowEvent::Touch(Touch {
                phase: TouchPhase::Started,
                ..
            }) => self.interaction_state = self.handle_cursor_down(),
            WindowEvent::Touch(Touch {
                phase: TouchPhase::Ended,
                ..
            }) => self.interaction_state = self.handle_cursor_up(),
            _ => {}
        }
    }
}

impl MandelbrotState {
    fn handle_cursor_move(&mut self, display: &mut Display, position: PhysicalPosition<f64>) {
        self.cursor_location = PhysicalPosition {
            x: position.x as f32,
            y: position.y as f32,
        };

        if self.cursor_location == self.last_cursor_location {
            return;
        }

        self.interaction_state = match self.interaction_state.clone() {
            InteractionState::PanningIdle { pressed_down_at: _ }
            | InteractionState::ZoomingIn
            | InteractionState::ZoomingOut
            | InteractionState::Panning => {
                let dx = (self.cursor_location.x - self.last_cursor_location.x)
                    / display.size.width as f32;
                let dy = (self.cursor_location.y - self.last_cursor_location.y)
                    / display.size.height as f32;

                self.uniforms.translation[0] -= dx * self.uniforms.scale;
                self.uniforms.translation[1] += dy * self.uniforms.scale;

                InteractionState::Panning
            }
            state => state,
        };

        self.last_cursor_location = self.cursor_location;
    }

    fn handle_cursor_down(&mut self) -> InteractionState {
        self.last_cursor_location = self.cursor_location;

        match self.interaction_state.clone() {
            InteractionState::Idle {
                pre_tapped_at: Some(pre_tapped_at),
            } => {
                if now_secs() - pre_tapped_at < 1f32 {
                    log::info!("Started zooming out at {}", now_secs());
                    InteractionState::ZoomingOut
                } else {
                    InteractionState::PanningIdle {
                        pressed_down_at: now_secs(),
                    }
                }
            }
            InteractionState::Idle { .. } => InteractionState::PanningIdle {
                pressed_down_at: now_secs(),
            },
            state => state,
        }
    }

    fn handle_cursor_up(&mut self) -> InteractionState {
        match self.interaction_state.clone() {
            InteractionState::PanningIdle { pressed_down_at } => {
                let elapsed = now_secs() - pressed_down_at;
                let pre_tapped_at = if elapsed < 0.3f32 {
                    Some(pressed_down_at)
                } else {
                    None
                };

                InteractionState::Idle { pre_tapped_at }
            }
            InteractionState::ZoomingIn
            | InteractionState::ZoomingOut
            | InteractionState::Panning => InteractionState::Idle {
                pre_tapped_at: None,
            },
            state => state,
        }
    }
}
