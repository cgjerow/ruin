use cgmath::SquareMatrix;
use std::collections::HashMap;
use winit::{event::WindowEvent, keyboard::KeyCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CameraMode {
    Perspective3D,
    Orthographic2D,
    Universal3D,
}

pub struct Camera3D {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    zoom: f32,
    mode: CameraMode,
}

impl Camera3D {
    pub fn new(width: u32, height: u32, mode: CameraMode) -> Self {
        Self {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 0.0, 5.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: Self::aspect_ratio(width, height),
            // this may need to update based on screen for 3D games
            fovy: 30.0,
            znear: 0.1,
            zfar: 100.0,
            zoom: 15.0, // how much of the world we see
            mode,
        }
    }

    fn aspect_ratio(width: u32, height: u32) -> f32 {
        width as f32 / height as f32
    }

    pub fn update_aspect_ratio(&mut self, width: u32, height: u32) {
        self.aspect = Self::aspect_ratio(width, height);
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        match self.mode {
            CameraMode::Perspective3D | CameraMode::Universal3D => {
                // move world to position/rotation of camera
                let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
                // make warp a little more dynamic based on aspect
                let fovy = (self.fovy / self.aspect.max(1.0)).clamp(30.0, 60.0);
                // warp scene to give effect of depth
                let proj =
                    cgmath::perspective(cgmath::Deg(fovy), self.aspect, self.znear, self.zfar);
                let opengl_to_wgpu_matrix_correction = cgmath::Matrix4::from_cols(
                    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
                    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
                    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
                    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
                );
                opengl_to_wgpu_matrix_correction * (proj * view)
            }
            CameraMode::Orthographic2D => {
                // Zoom controls how much of the world is visible: higher = see more
                let half_width = self.aspect * self.zoom;
                let half_height = 1.0 * self.zoom;

                let center_x = self.eye.x;
                let center_y = self.eye.y;

                let left = center_x - half_width;
                let right = center_x + half_width;
                let bottom = center_y - half_height;
                let top = center_y + half_height;

                let proj = cgmath::ortho(left, right, bottom, top, -1000.0, 1000.0);
                let view = cgmath::Matrix4::identity(); // no rotation or offset needed in view

                proj * view
            }
        }
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
    fn update_camera(&self, camera: &mut Camera3D);
}
