use glam::{Vec2, Vec3, Vec4};

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
