use std::f32::consts::PI;

use glam::{Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::Ray;

pub trait Camera {
    /// Returns the size of the viewport as a Vec2 (width, height).
    fn viewport_size(&self) -> Vec2;
    /// Returns the view matrix of the camera.
    fn view(&self) -> Mat4;
    /// Returns the projection matrix of the camera.
    fn projection(&self) -> Mat4 {
        glam::Mat4::perspective_rh_gl(self.fov(), self.aspect(), 0.1, 1024.0)
    }

    /// Field of view in radians
    fn fov(&self) -> f32 {
        PI / 3.0
    }

    /// Returns the combined view-projection matrix of the camera.
    fn view_projection(&self) -> Mat4 {
        self.projection() * self.view()
    }

    /// Returns the aspect ratio of the camera's viewport.
    fn aspect(&self) -> f32 {
        let viewport = self.viewport_size();
        if viewport.x == 0.0 {
            return 1.0;
        }
        viewport.x / viewport.y
    }

    /// Converts a world position (Vec3) to screen coordinates (Vec2).
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

    /// Returns the direction vector of the camera.
    fn direction(&self) -> Vec3;

    /// Returns the eye (position) of the camera.
    fn eye(&self) -> Vec3;

    /// Generates a ray from the camera through a given screen position.
    /// The screen position is given in pixel coordinates.
    fn screen_ray(&self, screen_pos: Vec2) -> Ray {
        let inv_vp = self.view_projection().inverse();
        let viewport = self.viewport_size();

        let ndc_x = (2.0 * screen_pos.x) / viewport.x - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.y) / viewport.y;

        let ndc_far = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);

        let world_far_h = inv_vp * ndc_far;

        let world_far = world_far_h.xyz() / world_far_h.w;

        let dir = world_far - self.eye();
        Ray {
            origin: self.eye(),
            dir,
        }
    }
}

pub mod orbital;
pub mod first_person;

pub use orbital::*;
pub use first_person::*;