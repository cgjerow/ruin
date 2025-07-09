use std::collections::HashMap;
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 1.0, 2.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: width / height,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // move world to position/rotation of camera
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // warp scene to give effect of depth
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        #[rustfmt::skip]
        let opengl_to_wgpu_matrix_correction: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
            cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
            cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
            cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
            cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
        );
        return opengl_to_wgpu_matrix_correction * (proj * view);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CameraAction {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    YawLeft,
    YawRight,
    PitchUp,
    PitchDown,
    RollLeft,
    RollRight,
}

pub struct CameraInputMap {
    key_map: HashMap<KeyCode, CameraAction>,
}

impl CameraInputMap {
    pub fn new() -> Self {
        Self {
            key_map: HashMap::new(),
        }
    }

    pub fn insert(mut self, key: KeyCode, action: CameraAction) -> Self {
        self.key_map.insert(key, action);
        self
    }

    pub fn get(&self, key: &KeyCode) -> Option<CameraAction> {
        self.key_map.get(key).copied()
    }
}

pub trait CameraController {
    fn process_events(&mut self, event: &WindowEvent) -> bool;
    fn update_camera(&self, camera: &mut Camera);
}
