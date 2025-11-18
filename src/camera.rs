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

#[derive(Default)]
/// An orbital camera that orbits around a target point.
pub struct OrbitalCamera {
    pub eye: Vec3,
    pub target: Vec3,
    pub viewport_size: Vec2,
}

impl OrbitalCamera {
    /// Zoom the camera in or out by moving the eye position closer to or further from the target.
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

    /// Rotate the camera around the target point by a given angle in radians.
    pub fn rotate_around(&mut self, r: f32) {
        let direction = self.eye - self.target;
        let rotation = Mat4::from_axis_angle(Vec3::Z, r);
        let rotated_direction = rotation.transform_vector3(direction);
        self.eye = self.target + rotated_direction;
    }

    /// Move the camera's eye and target positions by a given delta vector.
    /// The movement is relative to the camera's current orientation.
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

    /// Get the normalized direction vector from the eye to the target.
    pub fn direction(&self) -> Vec3 {
        let v = self.target - self.eye;
        v.normalize_or(Vec3::new(1.0, 0.0, 0.0))
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
    
    fn direction(&self) -> Vec3 {
        (self.target - self.eye).normalize_or_zero()
    }
    
    fn eye(&self) -> Vec3 {
        self.eye
    }
}


/// A first-person camera with pitch and yaw rotation.
pub struct FirstPersonCamera {
    pub eye: Vec3,
    pub yaw: f32,
    pub viewport_size: Vec2,
    pub pitch: f32,
}

impl Default for FirstPersonCamera {
    fn default() -> Self {
        Self {
            eye: Vec3::ZERO,
            yaw: 0.0,
            viewport_size: Vec2::new(800.0, 600.0),
            pitch: 0.0,
        }
    }
}

impl FirstPersonCamera {
    /// Move the camera forward/backward and left/right relative to its current orientation.
    pub fn move_self(&mut self, d: Vec3) {
        let forward = self.calculate_direction();
        let right = forward.cross(Vec3::Z).normalize_or(Vec3::new(1.0, 0.0, 0.0));
        let up = Vec3::Z;

        let movement = d.x * right + d.y * forward + d.z * up;
        self.eye += movement;
    }

    /// Rotate the camera around the Z axis by a given angle in radians.
    pub fn change_yaw(&mut self, angle: f32) {
        self.yaw += angle;
    }


    /// Rotate the camera around the Y axis by a given angle in radians.
    /// Pitch is clamped to prevent the camera from flipping over.
    pub fn change_pitch(&mut self, angle: f32) {
        // Update pitch and clamp it to prevent flipping
        self.pitch += angle;
        
        // Clamp pitch to prevent camera from flipping over
        // Limit to slightly less than 90 degrees to avoid gimbal lock
        let max_pitch = PI / 2.0 - 0.01;
        self.pitch = self.pitch.clamp(-max_pitch, max_pitch);
    }

    /// Get the current pitch angle in radians.
    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    /// Get the current yaw angle in radians.
    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    /// Calculate the direction vector from yaw and pitch.
    fn calculate_direction(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
        ).normalize()
    }
}

impl Camera for FirstPersonCamera {
    fn viewport_size(&self) -> Vec2 {
        self.viewport_size
    }

    fn view(&self) -> Mat4 {
        let up = Vec3::Z;
        Mat4::look_to_rh(self.eye, self.calculate_direction(), up)
        //Mat4::look_at_rh(self.eye, Default::default(), up)
    }

    fn direction(&self) -> Vec3 {
        self.calculate_direction()
    }

    fn eye(&self) -> Vec3 {
        self.eye
    }
}