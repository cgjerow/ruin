use cgmath::{ortho, InnerSpace, Matrix4, Vector2};

pub struct Camera2DConfig {
    pub zoom: f32, // how much of the world you see. larger = more
    pub initial_position: [f32; 2],
    pub look_ahead_smooth_factor: f32,
    pub look_ahead_distance: f32,
    pub look_ahead_lerp_speed: f32,
    pub screen_width: f32,
    pub screen_height: f32,
}

pub struct Camera2D {
    pub position: Vector2<f32>,
    pub zoom: f32,
    aspect_ratio: f32,
    smooth_factor: f32,
    look_ahead: f32,
    look_ahead_offset: Vector2<f32>,
    look_ahead_lerp_speed: f32,
    screen_width: f32,
    screen_height: f32,
    previous_velocity: Vector2<f32>,
}

impl Camera2D {
    pub fn new(config: &Camera2DConfig) -> Self {
        Self {
            position: Vector2::new(config.initial_position[0], config.initial_position[1]),
            zoom: config.zoom,
            aspect_ratio: config.screen_width / config.screen_height,
            smooth_factor: config.look_ahead_smooth_factor,
            look_ahead: config.look_ahead_distance,
            look_ahead_lerp_speed: config.look_ahead_lerp_speed,
            look_ahead_offset: Vector2::new(0.0, 0.0),
            previous_velocity: Vector2::new(0.0, 0.0),
            screen_width: config.screen_width,
            screen_height: config.screen_height,
        }
    }

    pub fn update_follow(&mut self, target: Vector2<f32>, velocity: Vector2<f32>) {
        let speed = velocity.magnitude();

        let dir_changed = (velocity.normalize().dot(self.previous_velocity.normalize())).is_nan()
            || velocity.normalize().dot(self.previous_velocity.normalize()) < 0.95;

        if speed > 0.1 && !dir_changed {
            let target_offset = velocity.normalize_to(self.look_ahead);
            self.look_ahead_offset +=
                (target_offset - self.look_ahead_offset) * self.look_ahead_lerp_speed;
        } else {
            self.look_ahead_offset *= 1.0 - self.look_ahead_lerp_speed;
        }

        if self.look_ahead_offset.magnitude() > self.look_ahead {
            self.look_ahead_offset = self.look_ahead_offset.normalize_to(self.look_ahead);
        }

        let predicted = target + self.look_ahead_offset;

        let to_predicted = predicted - self.position;
        self.position += to_predicted * self.smooth_factor;

        let after_move_to_predicted = predicted - self.position;
        if to_predicted.dot(after_move_to_predicted) < 0.0 {
            self.position = predicted;
        }

        if speed < 0.01 && (self.position - target).magnitude() < 0.1 {
            self.position = target;
        }

        self.previous_velocity = velocity;
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

    pub fn build_static_top_left_matrix(&self) -> Matrix4<f32> {
        ortho(0.0, self.screen_width, self.screen_height, 0.0, -1.0, 1.0)
    }

    pub fn build_static_center_matrix(&self) -> Matrix4<f32> {
        let half_w = self.screen_width / 2.0;
        let half_h = self.screen_height / 2.0;

        let left = -half_w;
        let right = half_w;
        let bottom = -half_h;
        let top = half_h;

        ortho(left, right, bottom, top, -1.0, 1.0)
    }
}
