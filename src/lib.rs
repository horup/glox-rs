mod camera;
mod shader;
mod ray;
pub use ray::*;
pub use camera::*;
mod draw_builder;
pub use draw_builder::*;

use glam::{Vec2, Vec3, Vec3Swizzles, Vec4};
use glow::{HasContext, Program};
use ply_rs as ply;
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

#[repr(C, packed)]
#[derive(Default, Clone, Copy, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub uv:[f32;2]
}

impl Vertex {
    pub fn new(position: Vec3, color: Vec4, uv:Vec2) -> Self {
        Self {
            position: position.to_array(),
            color: color.to_array(),
            uv:uv.to_array()
        }
    }
}

pub fn transform_vertices(vertices: &mut [Vertex], translation: Vec3, scaling: Vec3) -> impl Iterator<Item = Vertex> {
    vertices.iter().map(move |vertex| {
        let mut pos = Vec3::from(vertex.position);
        pos = pos * scaling + translation;
        Vertex {
            position: pos.to_array(),
            color: vertex.color,
            uv:vertex.uv
        }
    })
}   


pub fn floor_vertices(center: Vec3, color: Vec4) -> [Vertex; 6] {
    let half_size = 0.5;
    let c = center;

    [
        Vertex::new(Vec3::new(c.x - half_size, c.y + half_size, c.z), color, [0.0, 0.0].into()),
        Vertex::new(Vec3::new(c.x - half_size, c.y - half_size, c.z), color, [0.0, 1.0].into()),
        Vertex::new(Vec3::new(c.x + half_size, c.y - half_size, c.z), color, [1.0, 1.0].into()),
        Vertex::new(Vec3::new(c.x + half_size, c.y - half_size, c.z), color, [1.0, 1.0].into()),
        Vertex::new(Vec3::new(c.x + half_size, c.y + half_size, c.z), color, [1.0, 0.0].into()),
        Vertex::new(Vec3::new(c.x - half_size, c.y + half_size, c.z), color, [0.0, 0.0].into()),
    ]
}
pub fn ply_vertices(source:&str) -> Result<Vec<Vertex>, ()> {
    let mut vertices = Vec::new();
    let p = ply::parser::Parser::<ply::ply::DefaultElement>::new();
    let mut reader = std::io::Cursor::new(source);
    match p.read_ply(&mut reader) { 
        Err(_) => {
            return Err(());
        },
        Ok(p) => {
            let Some(faces) = p.payload.get("face") else { return Err(()); };
            let Some(vertex) = p.payload.get("vertex") else { return Err(()); };
            let vertex = |i:usize| -> Option<Vertex> {
                let Some(v) = &vertex.get(i) else { return None };
                let Some(ply_rs::ply::Property::Float(x)) = v.get("x") else { return None };
                let Some(ply_rs::ply::Property::Float(y)) = v.get("y") else { return None };
                let Some(ply_rs::ply::Property::Float(z)) = v.get("z") else { return None };
                let Some(ply_rs::ply::Property::UChar(r)) = v.get("red") else { return None };
                let Some(ply_rs::ply::Property::UChar(g)) = v.get("green") else { return None };
                let Some(ply_rs::ply::Property::UChar(b)) = v.get("blue") else { return None };
                let Some(ply_rs::ply::Property::UChar(a)) = v.get("alpha") else { return None };
                let r = *r as f32 / 255.0;
                let g = *g as f32 / 255.0;
                let b = *b as f32 / 255.0;
                let a = *a as f32 / 255.0;
                Some(Vertex { position: [*x, *y, *z], color: [r, g, b, a], uv: [0.0, 0.0] })
            };
            for face in faces.iter() {
                let Some(vertex_indices) = face.get("vertex_indices") else { return Err(()); };
                match vertex_indices {
                    ply_rs::ply::Property::ListUInt(index) => {
                        if index.len() != 3 {
                            continue;
                        }
                        let v0 = index[0] as usize;
                        let v1 = index[1] as usize;
                        let v2 = index[2] as usize;
                        let Some(v0) = vertex(v0) else { return Err(()); };
                        let Some(v1) = vertex(v1) else { return Err(()); };
                        let Some(v2) = vertex(v2) else { return Err(()); };
                        vertices.push(v0);
                        vertices.push(v1);
                        vertices.push(v2);
                    },
                    _=> {}
                }
            }
        }
    };

    Ok(vertices)
}

pub fn wall_vertices(bottom_center: Vec3, color: Vec4, normal: Vec3) -> [Vertex; 6] {
    let up = Vec3::new(0.0, 0.0, 1.0);
    let right = normal.cross(up).normalize();
    let half_width = 0.5;
    let height = 2.0;

    let top_center = bottom_center + up * height;

    let bl = bottom_center - right * half_width; // bottom left
    let br = bottom_center + right * half_width; // bottom right
    let tl = top_center - right * half_width; // top left
    let tr = top_center + right * half_width; // top right

    [
        Vertex::new(bl, color, [0.0, 1.0].into()),
        Vertex::new(br, color, [1.0, 1.0].into()),
        Vertex::new(tr, color, [1.0, 0.0].into()),
        Vertex::new(tr, color, [1.0, 0.0].into()),
        Vertex::new(tl, color, [0.0, 0.0].into()),
        Vertex::new(bl, color, [0.0, 1.0].into()),
    ]
}

pub fn line_vertices(start: Vec3, end: Vec3, width: f32, color: Vec4, camera_dir: Vec3) -> [Vertex; 6] {
    // Calculate line direction and perpendicular vector facing camera
    let line_dir = (end - start).normalize();
    let camera_dir_normalized = camera_dir.normalize();
    
    // Calculate right vector perpendicular to both line direction and camera direction
    let right = line_dir.cross(camera_dir_normalized).normalize();
    let half_width = width * 0.5;
    
    // Calculate quad corners
    let start_left = start - right * half_width;
    let start_right = start + right * half_width;
    let end_left = end - right * half_width;
    let end_right = end + right * half_width;
    
    [
        Vertex::new(start_left, color, [0.0, 0.0].into()),
        Vertex::new(start_right, color, [1.0, 0.0].into()),
        Vertex::new(end_right, color, [1.0, 1.0].into()),
        Vertex::new(end_right, color, [1.0, 1.0].into()),
        Vertex::new(end_left, color, [0.0, 1.0].into()),
        Vertex::new(start_left, color, [0.0, 0.0].into()),
    ]
}   

pub fn billboard_vertices(bottom_center: Vec3, color: Vec4, camera_dir: Vec3, scaling_factor: Vec2) -> [Vertex; 6] {
    let up = Vec3::new(0.0, 0.0, 1.0);
    let normal = camera_dir.xy() * -1.0;
    let normal = normal.extend(0.0);
    let right = normal.cross(up).normalize();
    let half_width = 0.5 * scaling_factor.x;
    let height = 1.0 * scaling_factor.y;

    let top_center = bottom_center + up * height;

    let bl = bottom_center - right * half_width; // bottom left
    let br = bottom_center + right * half_width; // bottom right
    let tl = top_center - right * half_width; // top left
    let tr = top_center + right * half_width; // top right

    [
        Vertex::new(bl, color, [0.0, 1.0].into()),
        Vertex::new(br, color, [1.0, 1.0].into()),
        Vertex::new(tr, color, [1.0, 0.0].into()),
        Vertex::new(tr, color, [1.0, 0.0].into()),
        Vertex::new(tl, color, [0.0, 0.0].into()),
        Vertex::new(bl, color, [0.0, 1.0].into()),
    ]
}
