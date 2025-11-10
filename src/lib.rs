mod camera;
mod shader;
mod ray;
mod vertices;
pub use vertices::*;
pub use ray::*;
pub use camera::*;
mod draw_builder;
pub use draw_builder::*;
mod vertex;
pub use vertex::*;

use glam::{Vec3, Vec4};
use glow::{HasContext, Program};
use std::mem::size_of;

#[derive(Default)]
pub struct Glox {
    program: Option<Program>,
    pub camera: Camera,
    pub vertex_array: Option<glow::VertexArray>,
    pub vertex_buffers: Vec<glow::Buffer>,
    pub vertex_buffer_current: usize,
    pub vertex_buffer_len: usize,
    pub vertex_buffer_vertex_index: usize,
}

impl Glox {
    pub fn init(&mut self, gl: &glow::Context) {
        self.camera.eye.z = 10.0;
        self.camera.eye.x = 0.0;
        self.camera.eye.y = -10.0;
        self.vertex_buffer_len = 1024 * 1024; // 1 million vertices
        unsafe {
            self.vertex_array = Some(
                gl.create_vertex_array()
                    .expect("failed to create vertex array"),
            );
            for _ in 0..3 {
                let vertex_buffer = gl.create_buffer().expect("failed to create buffer");
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
                gl.buffer_data_size(
                    glow::ARRAY_BUFFER,
                    self.vertex_buffer_len as i32 * size_of::<Vertex>() as i32,
                    glow::DYNAMIC_DRAW,
                );
                self.vertex_buffers.push(vertex_buffer);
            }

            let program = gl.create_program().expect("Cannot create program");
            let _: Vec<_> = shader::shader_sources()
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl
                        .create_shader(*shader_type)
                        .expect("Cannot create shader");
                    gl.shader_source(
                        shader,
                        &format!("{}\n{}", shader::shader_version(), shader_source),
                    );
                    gl.compile_shader(shader);
                    assert!(
                        gl.get_shader_compile_status(shader),
                        "Failed to compile {shader_type}: {}",
                        gl.get_shader_info_log(shader)
                    );
                    gl.attach_shader(program, shader);
                    shader
                })
                .collect();
            gl.link_program(program);

            self.program = Some(program);
        }
    }

    pub fn swap(&mut self) {
        self.vertex_buffer_vertex_index = 0;
        self.vertex_buffer_current = (self.vertex_buffer_current + 1) % self.vertex_buffers.len();
    }

    pub fn draw_grid(&mut self, gl: &glow::Context, grid_size: u32) {
        let mut builder = DrawBuilder::new(self, gl);
        let mut i = 0;
        for y in 0..grid_size {
            for x in 0..grid_size {
                i += 1;
                let center = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0);
                let c = 0.5;
                let color = Vec4::new(c, c, c, 1.0);
                let color = if i % 2 == 0 { color * 0.9 } else { color };

                let floor = floor_vertices(center, color);
                builder.push_vertices(&floor);
            }
        }

        builder.build();
    }

    pub fn draw_builder<'a>(&'a mut self, gl: &'a glow::Context) -> DrawBuilder<'a> {
        DrawBuilder::new(self, gl)
    }
}
