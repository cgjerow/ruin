use crate::camera_2d::Camera2D;
use cgmath::SquareMatrix;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform2D {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform2D {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update(&mut self, camera: &Camera2D) {
        self.view_proj = camera.build_matrix().into();
    }

    pub fn static_update(&mut self, camera: &Camera2D) {
        self.view_proj = camera.build_static_center_matrix().into();
    }
}
