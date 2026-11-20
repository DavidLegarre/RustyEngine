use winit::keyboard::KeyCode;

use crate::render::camera::Camera;

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        // Units per second. The default camera sits ~2.2 units from the target.
        Self::new(2.0)
    }
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    /// True while any movement key is held, so the app knows to keep animating.
    pub fn is_moving(&self) -> bool {
        self.is_forward_pressed
            || self.is_backward_pressed
            || self.is_left_pressed
            || self.is_right_pressed
    }

    /// Move the camera by `speed * delta_seconds`, so speed is units per second.
    pub fn update_camera(&self, camera: &mut Camera, delta_seconds: f32) {
        use cgmath::InnerSpace;
        let step = self.speed * delta_seconds;

        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when the camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > step {
            camera.eye += forward_norm * step;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * step;
        }

        let right = forward_norm.cross(camera.up);

        // Redo radius calc in case the forward/backward is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and the eye so
            // that it doesn't change. The eye, therefore, still
            // lies on the circle made by the target and eye.
            camera.eye = camera.target - (forward + right * step).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * step).normalize() * forward_mag;
        }
    }
}
