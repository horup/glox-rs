use glow::HasContext as _;

use crate::{Camera, Glox, Vertex};

pub struct DrawBuilder<'a> {
    renderer: &'a mut Glox,
    gl: &'a glow::Context,
    first: usize,
    count: usize,
}

impl<'a> DrawBuilder<'a> {
    const fn stride() -> i32 {
        std::mem::size_of::<Vertex>() as i32
    }

    pub fn bind_texture(&mut self, texture: Option<glow::Texture>) -> &mut Self {
        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, texture);
        }
        self
    }

    pub fn new(renderer: &'a mut Glox, gl: &'a glow::Context) -> Self {
        unsafe {
            let program = renderer.program.expect("no program");
            let vertex_buffer = renderer.vertex_buffers[renderer.vertex_buffer_current];
            let vertex_array = renderer.vertex_array.expect("no vertex_array");
            let first = renderer.vertex_buffer_vertex_index;
            gl.bind_vertex_array(Some(vertex_array));
            let texture = gl.create_texture().expect("failed to create texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            let data = vec![255u8, 255u8, 255u8, 255u8]; // white texture
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                1,
                1,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(&data)),
            );
            gl.generate_mipmap(glow::TEXTURE_2D);
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.use_program(Some(program));
            let view_projection = renderer.camera.view_projection();
            gl.uniform_matrix_4_f32_slice(
                gl.get_uniform_location(program, "view_projection").as_ref(),
                false,
                view_projection.as_ref(),
            );

            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, Self::stride(), 0);
            let offset = 3 * std::mem::size_of::<f32>() as i32;

            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, Self::stride(), offset);
            let offset = offset + 4 * std::mem::size_of::<f32>() as i32;

            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, Self::stride(), offset);

            Self {
                renderer,
                gl,
                first,
                count: 0,
            }
        }
    }
    pub fn push_vertices(&mut self, vertices: &[Vertex]) -> &mut Self {
        if self.renderer.vertex_buffer_vertex_index + vertices.len()
            >= self.renderer.vertex_buffer_len
        {
            self.renderer.vertex_buffer_vertex_index = 0;
        }

        unsafe {
            let vertex_data = std::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                std::mem::size_of_val(vertices),
            );

            self.gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                self.renderer.vertex_buffer_vertex_index as i32 * Self::stride(),
                vertex_data,
            );
            let count = vertices.len();
            self.renderer.vertex_buffer_vertex_index += count as usize;
            self.count += count;
        }

        self
    }

    pub fn build(self) {
        unsafe {
            self.gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
            self.gl.enable(glow::BLEND);

            self.gl
                .draw_arrays(glow::TRIANGLES, self.first as i32, self.count as i32);
        }
    }
}
