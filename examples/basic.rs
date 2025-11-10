use ggsdk::{GGAtlas, GGRunOptions, egui::Key};
use glam::{Vec2, Vec3, Vec4};
use glox::{Glox, OrbitalCamera};

#[derive(Default)]
struct App {
    pub glox: Glox,
    pub camera: OrbitalCamera
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
        self.camera.eye = Vec3::new(0.0, -10.0, 0.5);
        self.camera.target = Vec3::default();

        g.assets.load::<GGAtlas>("examples/wall_1x1.png", "wall");
    }

    fn update(&mut self, _: ggsdk::UpdateContext) {

    }

    fn update_glow(&mut self, g: ggsdk::UpdateContext) {
        let mut move_vec = Vec2::new(0.0, 0.0);
        let mut rot = 0.0;
        g.egui_ctx.input(|x|{
            let r = x.content_rect();
            self.camera.viewport_size = Vec2::new(r.width(), r.height());

            if x.key_down(Key::W) {
                move_vec.y = 1.0;
            }
            if x.key_down(Key::S) {
                move_vec.y = -1.0;
            }
            if x.key_down(Key::A) {
                move_vec.x = -1.0;
            }
            if x.key_down(Key::D) {
                move_vec.x = 1.0;
            }
            if x.key_down(Key::Q) {
                rot = -1.0;
            }
            if x.key_down(Key::E) {
                rot = 1.0;
            }
        });

        let d = g.dt;
        let speed = 10.0;
        let f = move_vec.extend(0.0) * d * speed;

        self.camera.rotate_self(rot * d);
        self.camera.eye += f;
    }

    fn paint_glow(&mut self, g: ggsdk::PaintGlowContext) {
        let Some(texture) = g.assets.get::<GGAtlas>("wall") else { return };
        let texture = g.painter.texture(texture.texture_id()).unwrap();
        let camera_dir= self.camera.direction();
        let gl = g.painter.gl();
        let mut draw = self.glox.draw_builder(gl, &self.camera);
        draw.bind_texture(Some(texture));
      //  draw.push_vertices(&glox::billboard_vertices(Default::default(), Vec4::splat(1.0), camera_dir, Vec2::splat(1.0)));
        draw.push_vertices(&glox::wall_vertices(Default::default(), 1.0, Vec4::splat(1.0), Vec3::new(0.0, 1.0, 0.0)));
        draw.finish();
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
