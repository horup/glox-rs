use glam::{Mat4, Vec2, Vec3};

use super::Camera;

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