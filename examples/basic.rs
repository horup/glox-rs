use std::collections::HashMap;

use ggsdk::{GGAtlas, GGRunOptions, egui::Key};
use glam::{Vec2, Vec3, Vec4};
use glox::{Glox, OrbitalCamera};

#[derive(Default)]
struct App {
    pub glox: Glox,
    pub orbital_camera: OrbitalCamera
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
        self.orbital_camera.eye = Vec3::new(0.0, -10.0, 10.0);
        self.orbital_camera.target = Vec3::default();

        g.assets.load::<GGAtlas>("examples/wall_1x1.png", "wall");
    }

    fn update(&mut self, _: ggsdk::UpdateContext) {

    }

    fn update_glow(&mut self, g: ggsdk::UpdateContext) {
        let mut move_vec = Vec2::new(0.0, 0.0);
        let mut rot = 0.0;
        g.egui_ctx.input(|x|{
            let r = x.content_rect();
            self.orbital_camera.viewport_size = Vec2::new(r.width(), r.height());

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

        self.orbital_camera.rotate_self(rot * d);
        self.orbital_camera.eye += f;
    }

    fn paint_glow(&mut self, g: ggsdk::PaintGlowContext) {
        let Some(texture) = g.assets.get::<GGAtlas>("wall") else { return };
        let texture = g.painter.texture(texture.texture_id()).unwrap();
        let camera_dir= self.orbital_camera.direction();
        let gl = g.painter.gl();

        // draw walls
        let mut walls = HashMap::new();
        let size = MAP.len();
        
        for y in 0..size {
            for x in 0..size {
                let cell = MAP[y][x];
                if cell == 1 {
                    let x_i = x as i32;
                    let y_i = y as i32;

                    // Check adjacent cells before adding walls
                    let has_top = y > 0 && MAP[y-1][x] == 1;
                    let has_right = x < size-1 && MAP[y][x+1] == 1;
                    let has_bottom = y < size-1 && MAP[y+1][x] == 1;
                    let has_left = x > 0 && MAP[y][x-1] == 1;

                    if !has_top {
                        walls.insert((x_i, y_i, true), ()); // top wall
                    }
                    if !has_right {
                        walls.insert((x_i + 1, y_i, false), ()); // right wall
                    }
                    if !has_bottom {
                        walls.insert((x_i, y_i + 1, true), ()); // bottom wall
                    }
                    if !has_left {
                        walls.insert((x_i, y_i, false), ()); // left wall
                    }
                }
            }
        }


        let mut draw = self.glox.draw_builder(gl, &self.orbital_camera);
        draw.bind_texture(Some(texture));

        for (x,y ,top) in walls.keys() {
            let n = match top {
                true => Vec3::new(0.0, 1.0, 0.0),
                false => Vec3::new(1.0, 0.0, 0.0),
            };
            let p = match top {
                true => Vec3::new(*x as f32 + 0.5, *y as f32, 0.0),
                false => Vec3::new(*x as f32, *y as f32 + 0.5, 0.0),
            };
            draw.push_vertices(&glox::wall_vertices(p, 1.0, Vec4::splat(1.0), n));
        }

      /*  for (x, y, top) in walls.drain(..) {
            let n = match top {
                true => Vec3::new(0.0, 1.0, 0.0),
                false => Vec3::new(1.0, 0.0, 0.0),
            };
            draw.push_vertices(&glox::wall_vertices((x, y, 0.0).into(), 1.0, Vec4::splat(1.0), n));

        }*/
      //  draw.push_vertices(&glox::billboard_vertices(Default::default(), Vec4::splat(1.0), camera_dir, Vec2::splat(1.0)));
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
