use std::f32::consts::PI;

use glam::{Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::Ray;

pub trait Camera {
    fn viewport_size(&self) -> Vec2;
    fn view(&self) -> Mat4;
    fn projection(&self) -> Mat4 {
        glam::Mat4::perspective_rh_gl(PI / 4.0, self.aspect(), 0.1, 1024.0)
    }

    fn view_projection(&self) -> Mat4 {
        self.projection() * self.view()
    }

    fn aspect(&self) -> f32 {
        let viewport = self.viewport_size();
        if viewport.x == 0.0 {
            return 1.0;
        }
        viewport.x / viewport.y
    }

    fn world_to_screen(&self, world_pos: Vec3) -> Vec2 {
        let p = self.view_projection() * world_pos.extend(1.0);
        let p = p / p.w;
        let p = p.xy();
        let viewport = self.viewport_size();
        Vec2::new(
            (p.x + 1.0) / 2.0 * viewport.x,
            (1.0 - p.y) / 2.0 * viewport.y,
        )
    }
}

#[derive(Default)]
pub struct OrbitalCamera {
    pub eye: Vec3,
    pub target: Vec3,
    pub viewport_size: Vec2,
}

impl OrbitalCamera {
    pub fn zoom_self(&mut self, delta: f32) {
        let direction = (self.target - self.eye).normalize_or(Vec3::new(0.0, 0.0, 1.0));
        let new_eye = self.eye + direction * delta;
        let a = self.target - new_eye;
        let a = a.normalize_or_zero();
        if direction.dot(a) > 0.0 {
            self.eye = new_eye;
        }
        let d = self.target - self.eye;
        let max = 3.0;
        if d.length() < max {
            self.eye = self.target - direction * max;
        }
    }

    pub fn rotate_self(&mut self, r: f32) {
        let direction = self.eye - self.target;
        let rotation = Mat4::from_axis_angle(Vec3::Z, r);
        let rotated_direction = rotation.transform_vector3(direction);
        self.eye = self.target + rotated_direction;
    }

    pub fn move_self(&mut self, d: Vec3) {
        // Calculate forward direction ignoring vertical (Z) component
        let mut forward = self.target - self.eye;
        forward.z = 0.0;
        let forward = forward.normalize_or(Vec3::new(0.0, 1.0, 0.0));

        let right = forward
            .cross(Vec3::Z)
            .normalize_or(Vec3::new(1.0, 0.0, 0.0));
        let up = Vec3::Z;

        let movement = d.x * right + d.y * forward + d.z * up;

        self.eye += movement;
        self.target += movement;
    }

    pub fn direction(&self) -> Vec3 {
        let v = self.target - self.eye;
        v.normalize_or(Vec3::new(1.0, 0.0, 0.0))
    }

    pub fn screen_ray(&self, screen_pos: Vec2) -> Ray {
        let inv_vp = self.view_projection().inverse();
        let viewport = self.viewport_size();

        let ndc_x = (2.0 * screen_pos.x) / viewport.x - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.y) / viewport.y;

        let ndc_far = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);

        let world_far_h = inv_vp * ndc_far;

        let world_far = world_far_h.xyz() / world_far_h.w;

        let dir = (world_far - self.eye).normalize();
        Ray {
            origin: self.eye,
            dir,
        }
    }
}

impl Camera for OrbitalCamera {
    fn viewport_size(&self) -> Vec2 {
        self.viewport_size
    }

    fn view(&self) -> Mat4 {
        let v = self.target - self.eye;
        let forward = v.normalize_or(Vec3::new(0.0, 0.0, 1.0));
        let up = Vec3::new(0.0, 0.0, 1.0);
        let right = forward.cross(up);
        let up = right.cross(forward);

        Mat4::look_at_rh(self.eye, self.target, up)
    }
}
