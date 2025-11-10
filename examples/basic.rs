use ggsdk::GGRunOptions;
use glam::{Vec3, Vec4};
use glox::{Camera, Glox};

#[derive(Default)]
struct App {
    pub glox:Glox
}

impl ggsdk::GGApp for App {
    fn init(&mut self, g: ggsdk::InitContext) {
        self.glox.init(g.gl);
    }

    fn update(&mut self, g: ggsdk::UpdateContext) {
       
    }

    fn paint_glow(&mut self, g:ggsdk::PaintGlowContext) {
        let gl = g.painter.gl();
        let mut draw = self.glox.draw_builder(gl);
        draw.push_vertices(&glox::floor_vertices(Vec3::new(0.0, 0.0, 0.0), Vec4::new(1.0, 1.0, 1.0, 1.0)));
        draw.build();
    }
}

fn main() {
    let app = App::default();
    ggsdk::GGEngine::run(app, GGRunOptions {
        ..Default::default()
    });
}