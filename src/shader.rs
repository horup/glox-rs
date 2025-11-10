pub fn shader_version() -> &'static str {
    if cfg!(target_arch = "wasm32") {
        "#version 300 es"
    } else {
        "#version 330"
    }
}

pub fn shader_sources() -> [(u32, &'static str);2] {
    let (vertex_shader_source, fragment_shader_source) = (
        r#"
                layout(location = 0) in vec3 aPos;   
                layout(location = 1) in vec4 aColor;
                layout(location = 2) in vec2 aUv;
                out vec4 vertexColor;
                out vec2 uv;
                uniform mat4 view_projection;
                void main() {
                    gl_Position = view_projection * vec4(aPos, 1.0);
                    uv = aUv;
                    vertexColor = aColor;
                }
            "#,
        r#"
                precision mediump float;
                uniform sampler2D tex;
                in vec4 vertexColor;
                in vec2 uv;
                out vec4 fragColor;
                void main() {
                    vec4 color = texture(tex, uv) * vertexColor;
                    if (color.a == 0.0) {
                        discard;
                    }
                    fragColor = color;
                }
            "#,
    );

    
    [
        (glow::VERTEX_SHADER, vertex_shader_source),
        (glow::FRAGMENT_SHADER, fragment_shader_source),
    ]
}
