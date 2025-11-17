use std::collections::HashMap;

use ggsdk::{
    GGAtlas, GGRunOptions,
    egui::{self, Align2, Color32, FontId, Key, LayerId},
};
use glam::{Vec2, Vec3, Vec4};
use glow::HasContext;
use glox::{Camera, FirstPersonCamera, Glox, OrbitalCamera};

#[derive(PartialEq, Eq)]
pub enum ChosenCamera {
    Orbital,
    FirstPerson,
}
impl Default for ChosenCamera {
    fn default() -> Self {
        Self::FirstPerson
    }
}

#[derive(Default)]
struct App {
    pub glox: Glox,
    pub orbital_camera: OrbitalCamera,
    pub fps_camera: FirstPersonCamera,
    pub chosen_camera: ChosenCamera,
    pub cursor_grab: bool,
}

static MAP: [[u8; 8]; 8] = [
    [1, 1, 1, 1, 1, 1, 1, 1],
    [1, 2, 0, 0, 1, 0, 2, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 4, 0, 3, 1, 0, 0, 1],
    [1, 1, 1, 1, 1, 0, 0, 1],
    [1, 3, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 5, 1],
    [1, 1, 1, 1, 1, 1, 1, 1],
];

impl ggsdk::GGApp for App {
    fn init(&mut self, g: ggsdk::InitContext) {
        self.glox.init(g.gl);
        self.orbital_camera.eye = Vec3::new(0.0, -10.0, 10.0);
        self.orbital_camera.target = Vec3::default();
        self.fps_camera.eye = Vec3::new(2.5, 2.5, 0.5);

        g.assets
            .load::<GGAtlas>("examples/imgs/wall_1x1.png", "wall");
        g.assets
            .load::<GGAtlas>("examples/imgs/cross_1x1.png", "cross");
        g.assets
            .load::<GGAtlas>("examples/imgs/lamp_1x1.png", "lamp");
        g.assets
            .load::<GGAtlas>("examples/imgs/plant_1x1.png", "plant");
        g.assets
            .load::<GGAtlas>("examples/imgs/chairs_1x1.png", "chairs");
        g.assets
            .load::<GGAtlas>("examples/imgs/player_1x1.png", "player");
    }

    fn update(&mut self, g: ggsdk::UpdateContext) {
        //g.egui_ctx.send_viewport_cmd_to(id, ViewP);
        let painter = g.egui_ctx.layer_painter(LayerId::background());
        painter.text(
            (0.0, 0.0).into(),
            Align2::LEFT_TOP,
            "Press Tab to switch focus",
            FontId::default(),
            Color32::WHITE,
        );
        if self.cursor_grab {
            return;
        }
        egui::Window::new("Controls").show(g.egui_ctx, |ui| {
            ui.radio_value(
                &mut self.chosen_camera,
                ChosenCamera::Orbital,
                "Orbital Camera",
            );
            ui.radio_value(
                &mut self.chosen_camera,
                ChosenCamera::FirstPerson,
                "First Person Camera",
            );
        });
    }

    fn update_glow(&mut self, g: ggsdk::UpdateContext) {
        let mut move_vec = Vec2::new(0.0, 0.0);
        let mut rot = 0.0;
        let mut pointer_delta = Vec2::new(0.0, 0.0);

        g.egui_ctx
            .send_viewport_cmd(egui::ViewportCommand::CursorGrab(egui::CursorGrab::None));

        let mut grabed_changed = false;
        g.egui_ctx.input(|x| {
            let r = x.content_rect();
            self.orbital_camera.viewport_size = Vec2::new(r.width(), r.height());
            self.fps_camera.viewport_size = Vec2::new(r.width(), r.height());

            if x.key_pressed(Key::Tab) {
                grabed_changed = true;
                self.cursor_grab = !self.cursor_grab;
            }

            if self.cursor_grab == false {
                return;
            }

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

            let delta = x.pointer.delta();
            pointer_delta = Vec2::new(delta.x, delta.y);
        });

        if grabed_changed {
            match self.cursor_grab {
                true => {
                    g
                    .egui_ctx
                    .send_viewport_cmd(egui::ViewportCommand::CursorGrab(egui::CursorGrab::Confined));
                    g.egui_ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(false));
                },
                false => {
                    g
                    .egui_ctx
                    .send_viewport_cmd(egui::ViewportCommand::CursorGrab(egui::CursorGrab::None));
                    g.egui_ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(true));
                },
            };
        }

        let d = g.dt;
        let speed = 10.0;
        let f = move_vec.extend(0.0) * d * speed;

        match self.chosen_camera {
            ChosenCamera::Orbital => {
                self.orbital_camera.move_self(f);
                self.orbital_camera.rotate_around(rot * d * 2.0);
            }
            ChosenCamera::FirstPerson => {
                self.fps_camera.move_self(f / 2.0);
                self.fps_camera.rotate_z(-rot * d * 4.0);

                let senitivity = 0.01;
                self.fps_camera.rotate_z(pointer_delta.x * -senitivity);
            }
        }
    }

    fn paint_glow(&mut self, g: ggsdk::PaintGlowContext) {
        let camera: &dyn Camera = match self.chosen_camera {
            ChosenCamera::Orbital => &self.orbital_camera,
            ChosenCamera::FirstPerson => &self.fps_camera,
        };
        let Some(texture) = g.assets.get::<GGAtlas>("wall") else {
            return;
        };
        let texture = g.painter.texture(texture.texture_id()).unwrap();
        let camera_dir = camera.direction();
        let gl = g.painter.gl();
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }
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
                    let has_top = y > 0 && MAP[y - 1][x] == 1;
                    let has_right = x < size - 1 && MAP[y][x + 1] == 1;
                    let has_bottom = y < size - 1 && MAP[y + 1][x] == 1;
                    let has_left = x > 0 && MAP[y][x - 1] == 1;

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

        // draw all walls
        let mut draw = self.glox.draw_builder(gl, camera);
        draw.push_vertices(&glox::plane_vertices(
            Default::default(),
            Vec4::new(0.4, 0.4, 0.4, 1.0),
            1024.0,
        ));
        draw.finish();

        let mut draw = self.glox.draw_builder(gl, camera);
        draw.bind_texture(Some(texture));

        for (x, y, top) in walls.keys() {
            let n = match top {
                true => Vec3::new(0.0, 1.0, 0.0),
                false => Vec3::new(1.0, 0.0, 0.0),
            };
            let c = 0.5;
            let color = match top {
                true => Vec4::new(1.0, 1.0, 1.0, 1.0),
                false => Vec4::new(c, c, c, 1.0),
            };
            let p = match top {
                true => Vec3::new(*x as f32 + 0.5, *y as f32, 0.0),
                false => Vec3::new(*x as f32, *y as f32 + 0.5, 0.0),
            };

            draw.push_vertices(&glox::wall_vertices(p, 1.0, color, n));
        }
        draw.finish();

        // draw top of block
        let mut draw = self.glox.draw_builder(gl, camera);
        draw.bind_texture(Some(texture));
        for y in 0..size {
            for x in 0..size {
                if MAP[y][x] != 1 {
                    continue;
                }
                let p = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 1.0);
                let color = Vec4::new(0.2, 0.2, 0.2, 1.0);
                draw.push_vertices(&glox::floor_vertices(p, color));
            }
        }
        draw.finish();

        // draw some sprites / billboards
        for y in 0..size {
            for x in 0..size {
                let mut draw = self.glox.draw_builder(gl, camera);
                //draw.bind_texture(Some(texture));
                let id = MAP[y][x];
                let texture = match id {
                    2 => "cross",
                    3 => "plant",
                    4 => "chairs",
                    5 => "lamp",
                    _ => {
                        continue;
                    }
                };
                if let Some(atlas) = g.assets.get::<GGAtlas>(texture) {
                    let texture = g.painter.texture(atlas.texture_id()).unwrap();
                    draw.bind_texture(texture.into());
                }
                let p = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0);
                draw.push_vertices(&glox::billboard_vertices(
                    p,
                    Vec4::splat(1.0),
                    camera_dir,
                    Vec2::splat(1.0),
                ));
                draw.finish();
            }
        }

        // draw fps camera pos if orbital camera
        if self.chosen_camera == ChosenCamera::Orbital {
            let p = self.fps_camera.eye;
            let mut draw = self.glox.draw_builder(gl, camera);
            if let Some(atlas) = g.assets.get::<GGAtlas>("player") {
                let texture = g.painter.texture(atlas.texture_id()).unwrap();
                draw.bind_texture(texture.into());
            }
            draw.push_vertices(&glox::billboard_vertices(
                Vec3::new(p.x, p.y, 0.0),
                Vec4::splat(1.0),
                camera_dir,
                Vec2::splat(1.0),
            ));
            draw.finish();
        }

        self.glox.swap();
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
