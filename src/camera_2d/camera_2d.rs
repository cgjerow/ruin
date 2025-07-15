use cgmath::{ortho, Matrix4, Vector2};

pub struct Camera2D {
    pub position: Vector2<f32>,
    pub zoom: f32, // how much of the world you see. larger = more
    pub aspect_ratio: f32,
    pub smooth_factor: f32,
}

impl Camera2D {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            position: Vector2::new(0.0, 0.0),
            zoom: 15.0,
            aspect_ratio: width as f32 / height as f32,
            smooth_factor: 0.1,
        }
    }

    pub fn update_follow(&mut self, target: Vector2<f32>) {
        let to_target = target - self.position;
        self.position += to_target * self.smooth_factor;
    }

    pub fn update_aspect_ratio(&mut self, width: u32, height: u32) {
        self.aspect_ratio = width as f32 / height as f32;
    }

    pub fn build_matrix(&self) -> Matrix4<f32> {
        let half_width = self.aspect_ratio * self.zoom;
        let half_height = self.zoom;

        let left = self.position.x - half_width;
        let right = self.position.x + half_width;
        let bottom = self.position.y - half_height;
        let top = self.position.y + half_height;

        // z: -1 to 1, because we're not using 3D
        ortho(left, right, bottom, top, -1.0, 1.0)
    }
}
