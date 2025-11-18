use std::f32::consts::PI;

use glam::{Mat4, Vec2, Vec3};

use super::Camera;

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
        let right = forward
            .cross(Vec3::Z)
            .normalize_or(Vec3::new(1.0, 0.0, 0.0));
        let up = Vec3::Z;

        let movement = d.x * right + d.y * forward + d.z * up;
        self.eye += movement;
    }

    /// Move the camera forward/backward and left/right relative to its horizontal orientation only.
    /// This ignores pitch and only uses the yaw for movement direction.
    pub fn move_self_horizontal(&mut self, d: Vec3) {
        let forward = self.forward();
        let right = forward
            .cross(Vec3::Z)
            .normalize_or(Vec3::new(1.0, 0.0, 0.0));
        let up = Vec3::Z;

        let movement = d.x * right + d.y * forward + d.z * up;
        self.eye += movement;
    }

    /// Set the camera to look towards a specific direction vector.
    /// The direction vector will be normalized automatically.
    pub fn look_to(&mut self, direction: Vec3) {
        let dir = direction.normalize();
        
        // Calculate yaw (rotation around Z axis)
        // atan2(y, x) gives us the angle in the XY plane
        self.yaw = dir.y.atan2(dir.x);
        
        // Calculate pitch (rotation around the horizontal plane)
        // asin(z) gives us the vertical angle
        self.pitch = dir.z.asin();
        
        // Clamp pitch to prevent camera from flipping over
        let max_pitch = PI / 2.0 - 0.01;
        self.pitch = self.pitch.clamp(-max_pitch, max_pitch);
    }

    /// Set the camera to look at a specific point in space.
    /// This calculates the direction from the camera's eye position to the target point.
    pub fn look_at(&mut self, target: Vec3) {
        let direction = target - self.eye;
        self.look_to(direction);
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

    /// Get the forward direction vector of the camera.
    /// This is the normalized direction the camera is currently facing.
    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos(),
            self.yaw.sin(),
            0.0,
        ).normalize()
    }

    /// Calculate the direction vector from yaw and pitch.
    fn calculate_direction(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
        )
        .normalize()
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