use crate::camera::{Camera, CameraAction, CameraController, CameraInputMap}; // or `super::` if you prefer
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

pub struct TwoDimensionalCameraController {
    speed: f32,
    locked: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl TwoDimensionalCameraController {
    pub fn new(locked: bool, speed: f32) -> Self {
        Self {
            locked,
            speed: speed / 1000.0,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
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
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        use cgmath::{InnerSpace, Vector3};

        let forward = (camera.target - camera.eye).normalize();
        let right = forward.cross(camera.up).normalize();

        let mut movement = Vector3::new(0.0, 0.0, 0.0);

        if self.is_forward_pressed {
            movement += camera.up * self.speed;
        }
        if self.is_backward_pressed {
            movement -= camera.up * self.speed;
        }
        if self.is_right_pressed {
            movement += right * self.speed;
        }
        if self.is_left_pressed {
            movement -= right * self.speed;
        }

        camera.eye += movement;
        camera.target += movement;
    }
}

pub struct ThreeDimensionalCameraController {
    locked: bool,
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl ThreeDimensionalCameraController {
    pub fn new(locked: bool, speed: f32) -> Self {
        Self {
            locked,
            speed: speed / 1000.0,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
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
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        /*
         * use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when the camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        // Redo radius calc in case the forward/backward is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and the eye so
            // that it doesn't change. The eye, therefore, still
            // lies on the circle made by the target and eye.
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
        */
        use cgmath::{InnerSpace, Vector3};

        let forward = (camera.target - camera.eye).normalize();
        let right = forward.cross(camera.up).normalize();

        let mut movement = Vector3::new(0.0, 0.0, 0.0);

        if self.is_forward_pressed {
            movement += forward * self.speed;
        }
        if self.is_backward_pressed {
            movement -= forward * self.speed;
        }
        if self.is_right_pressed {
            movement += right * self.speed;
        }
        if self.is_left_pressed {
            movement -= right * self.speed;
        }

        // Apply movement to both eye and target to maintain direction
        camera.eye += movement;
        camera.target += movement;
    }
}

impl CameraController for TwoDimensionalCameraController {
    fn process_events(&mut self, event: &WindowEvent) -> bool {
        self.process_events(event)
    }

    fn update_camera(&self, camera: &mut Camera) {
        self.update_camera(camera)
    }
}

impl CameraController for ThreeDimensionalCameraController {
    fn process_events(&mut self, event: &WindowEvent) -> bool {
        self.process_events(event)
    }

    fn update_camera(&self, camera: &mut Camera) {
        self.update_camera(camera)
    }
}
