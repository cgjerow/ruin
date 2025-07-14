use cgmath::Rotation;
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::PhysicalKey,
};

use crate::camera::{Camera, CameraAction, CameraController, CameraInputMap};
use std::collections::HashMap;

pub struct UniversalCameraController {
    speed: f32,
    sensitivity: f32,
    locked: bool,
    input_map: CameraInputMap,
    // Options
    enable_rotation: bool,
    enable_roll: bool,
    enable_flight_mode: bool, // disables "lock to y-up"
    input_state: HashMap<CameraAction, bool>,
}

impl UniversalCameraController {
    pub fn new(locked: bool, speed: f32, sensitivity: f32, input_map: CameraInputMap) -> Self {
        Self {
            locked,
            speed: speed / 1000.0,
            sensitivity: sensitivity / 1000.0,
            input_map,
            enable_roll: true,
            enable_rotation: true,
            enable_flight_mode: true,
            input_state: HashMap::new(),
        }
    }

    pub fn disable_roll(mut self) -> Self {
        self.enable_roll = false;
        self
    }

    pub fn enable_roll(mut self) -> Self {
        self.enable_roll = true;
        self
    }

    pub fn disable_rotation(mut self) -> Self {
        self.enable_rotation = false;
        self
    }

    pub fn enable_rotation(mut self) -> Self {
        self.enable_rotation = true;
        self
    }

    pub fn disable_flight_mode(mut self) -> Self {
        self.enable_roll = false;
        self
    }

    pub fn enable_flight_mode(mut self) -> Self {
        self.enable_roll = true;
        self
    }
}

impl CameraController for UniversalCameraController {
    fn process_events(&mut self, event: &WindowEvent) -> bool {
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
                if let Some(action) = self.input_map.get(keycode) {
                    let is_pressed = *state == ElementState::Pressed;
                    self.input_state.insert(action, is_pressed);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn update_camera(&self, camera: &mut Camera) {
        use cgmath::{InnerSpace, Quaternion, Rad, Rotation3, Vector3};

        let forward = (camera.target - camera.eye).normalize();
        let right = forward.cross(camera.up).normalize();
        let up = if self.enable_flight_mode {
            camera.up
        } else {
            Vector3::unit_y()
        };

        let mut movement = Vector3::new(0.0, 0.0, 0.0);

        if self.input_state.get(&CameraAction::MoveForward) == Some(&true) {
            movement += forward * self.speed;
        }
        if self.input_state.get(&CameraAction::MoveBackward) == Some(&true) {
            movement -= forward * self.speed;
        }
        if self.input_state.get(&CameraAction::MoveRight) == Some(&true) {
            movement += right * self.speed;
        }
        if self.input_state.get(&CameraAction::MoveLeft) == Some(&true) {
            movement -= right * self.speed;
        }
        if self.input_state.get(&CameraAction::MoveUp) == Some(&true) {
            movement += up * self.speed;
        }
        if self.input_state.get(&CameraAction::MoveDown) == Some(&true) {
            movement -= up * self.speed;
        }

        camera.eye += movement;
        camera.target += movement;

        if self.enable_rotation {
            let mut pitch = 0.0;
            let mut yaw = 0.0;
            let mut roll = 0.0;

            if self.input_state.get(&CameraAction::YawLeft) == Some(&true) {
                yaw += self.sensitivity;
            }
            if self.input_state.get(&CameraAction::YawRight) == Some(&true) {
                yaw -= self.sensitivity;
            }
            if self.input_state.get(&CameraAction::PitchUp) == Some(&true) {
                pitch += self.sensitivity;
            }
            if self.input_state.get(&CameraAction::PitchDown) == Some(&true) {
                pitch -= self.sensitivity;
            }
            if self.enable_roll {
                if self.input_state.get(&CameraAction::RollLeft) == Some(&true) {
                    roll -= self.sensitivity;
                }
                if self.input_state.get(&CameraAction::RollRight) == Some(&true) {
                    roll += self.sensitivity;
                }
            }

            let yaw_quat = Quaternion::from_axis_angle(up, Rad(yaw));
            let pitch_quat = Quaternion::from_axis_angle(right, Rad(pitch));
            let roll_quat = Quaternion::from_axis_angle(forward, Rad(roll));

            let rotation = if self.enable_roll {
                roll_quat * pitch_quat * yaw_quat
            } else {
                pitch_quat * yaw_quat
            };

            let new_forward = rotation.rotate_vector(forward);
            camera.target = camera.eye + new_forward;
            camera.up = rotation.rotate_vector(camera.up).normalize();
        }
    }
}
