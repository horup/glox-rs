use ggsdk::{GGAtlas, GGRunOptions};
use glam::{Vec2, Vec3, Vec4};
use glox::Glox;

#[derive(Default)]
struct App {
    pub glox: Glox,
}

static MAP:[[u8;8];8] = [
    [1,1,1,1,1,1,1,1],
    [1,0,0,0,1,0,0,1],
    [1,0,0,0,0,0,0,1],
    [1,0,0,0,1,0,0,1],
    [1,1,1,1,1,0,0,1],
    [1,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,1],
    [1,1,1,1,1,1,1,1]];

impl ggsdk::GGApp for App {
    fn init(&mut self, g: ggsdk::InitContext) {
        self.glox.init(g.gl);
        self.glox.camera.eye = Vec3::new(0.0, -10.0, 0.5);
        self.glox.camera.target = Vec3::default();

        g.assets.load::<GGAtlas>("examples/wall_1x1.png", "wall");
    }

    fn update(&mut self, _: ggsdk::UpdateContext) {

    }

    fn update_glow(&mut self, g: ggsdk::UpdateContext) {
        g.egui_ctx.input(|x|{
            let r = x.content_rect();
            self.glox.camera.viewport_size = Vec2::new(r.width(), r.height());
        });
    }

    fn paint_glow(&mut self, g: ggsdk::PaintGlowContext) {
        let Some(texture) = g.assets.get::<GGAtlas>("wall") else { return };
        let texture = g.painter.texture(texture.texture_id()).unwrap();
        let camera_dir= self.glox.camera.direction();
        let gl = g.painter.gl();
        let mut draw = self.glox.draw_builder(gl);
        draw.bind_texture(Some(texture));
        draw.push_vertices(&glox::billboard_vertices(Default::default(), Vec4::splat(1.0), camera_dir, Vec2::splat(1.0)));
        draw.build();
    }
}

fn main() {
    let app = App::default();
    ggsdk::GGEngine::run(
        app,
        GGRunOptions {
            ..Default::default()
        },
    );
}
