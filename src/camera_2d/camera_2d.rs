use cgmath::{Matrix4, Point3, Vector3};

pub struct Camera2D {
    pub center: [f32; 2],
    pub zoom: f32,
    pub lookahead: f32,
    pub smoothing: f32,
    pub eye_z: f32, // where the camera "sits" in z
    pub near: f32,
    pub far: f32,
    pub aspect_ratio: f32,
}

impl Camera2D {
    pub fn update_follow(&mut self, target: [f32; 2], velocity: [f32; 2]) {
        let desired_x = target[0] + velocity[0] * self.lookahead;
        let desired_y = target[1] + velocity[1] * self.lookahead;

        self.center[0] += (desired_x - self.center[0]) * self.smoothing;
        self.center[1] += (desired_y - self.center[1]) * self.smoothing;
    }

    pub fn build_view_proj_matrix(&self, aspect: f32) -> Matrix4<f32> {
        let half_w = self.zoom * aspect;
        let half_h = self.zoom;

        let left = self.center[0] - half_w;
        let right = self.center[0] + half_w;
        let bottom = self.center[1] - half_h;
        let top = self.center[1] + half_h;

        // View: camera at (center.x, center.y, eye_z), looking at Z = 0
        let eye = Point3::new(self.center[0], self.center[1], self.eye_z);
        let target = Point3::new(self.center[0], self.center[1], 0.0);
        let up = Vector3::unit_y();

        let view = Matrix4::look_at_rh(eye, target, up);
        let proj = cgmath::ortho(left, right, bottom, top, self.near, self.far);

        proj * view
    }

    pub fn update_aspect_ratio(&mut self, width: u32, height: u32) {
        self.aspect_ratio = width as f32 / height as f32;
    }
}
