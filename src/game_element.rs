use std::{collections::HashMap, u16};

use crate::texture::Texture;

// Maybe?
pub trait DrawableEntity {
    fn update(&mut self, dt: f32);
    fn get_position(&self) -> [f32; 3];
    fn get_uv_coords(&self) -> Option<[[f32; 2]; 4]>;
}

#[derive(Debug, Clone)]
pub struct SpriteFrame {
    pub uv_coords: [[f32; 2]; 4], // bottom-left, bottom-right, top-right, top-left
    pub duration: f32,
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub frames: Vec<SpriteFrame>,
    pub looped: bool,
}

impl Animation {
    pub fn from_lua_table(
        table: mlua::Table,
        sprite_sheet_width: u16,
        sprite_sheet_height: u16,
    ) -> Self {
        let looped: bool = table.get("looped").unwrap_or(true);

        let frames_table: mlua::Table = table
            .get("frames")
            .expect("Missing 'frames' table in animation");

        let tex_w = sprite_sheet_width as f32;
        let tex_h = sprite_sheet_height as f32;

        let mut frames = Vec::new();
        for pair in frames_table.sequence_values::<mlua::Table>() {
            if let Ok(frame_data) = pair {
                let x: f32 = frame_data.get("x").unwrap_or(0.0);
                let y: f32 = frame_data.get("y").unwrap_or(0.0);
                let w: f32 = frame_data.get("width").unwrap_or(1.0);
                let h: f32 = frame_data.get("height").unwrap_or(1.0);
                let duration: f32 = frame_data.get("duration").unwrap_or(1.0);

                // Convert to UVs (and optionally flip Y if needed)
                let u0 = x / tex_w;
                let v0 = y / tex_h;
                let u1 = (x + w) / tex_w;
                let v1 = (y + h) / tex_h;

                // WGPU uses origin at top-left by default. Flip V if needed.
                let uv_coords = [
                    [u0, v1], // bottom-left
                    [u1, v1], // bottom-right
                    [u1, v0], // top-right
                    [u0, v0], // top-left
                ];

                frames.push(SpriteFrame {
                    uv_coords,
                    duration,
                });
            }
        }

        Animation { frames, looped }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum VisualState {
    Idle,
    Walking,
    Running,
    Jumping,
    Landing,
    Dying,
    Colliding,
    Custom(String),
}

impl From<String> for VisualState {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Idle" => VisualState::Idle,
            "Walking" => VisualState::Walking,
            "Running" => VisualState::Running,
            "Jumping" => VisualState::Jumping,
            "Landing" => VisualState::Landing,
            "Dying" => VisualState::Dying,
            "Colliding" => VisualState::Colliding,
            other => VisualState::Custom(other.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatefulElement {
    pub id: String,
    pub position: [f32; 3],
    pub size: [f32; 2],
    pub state: VisualState,
    pub animations: HashMap<VisualState, Animation>,
    pub current_frame: usize,
    pub frame_timer: f32,
    pub sprite_sheet: Texture,
}

impl StatefulElement {
    pub fn update(&mut self, dt: f32) {
        if let Some(anim) = self.animations.get(&self.state) {
            if anim.frames.is_empty() {
                return;
            }

            self.frame_timer += dt;

            let current = self.current_frame.min(anim.frames.len() - 1);
            let frame_duration = anim.frames[current].duration;

            if self.frame_timer >= frame_duration {
                self.frame_timer -= frame_duration; // carry over extra time
                self.current_frame += 1;

                if self.current_frame >= anim.frames.len() {
                    self.current_frame = if anim.looped {
                        0
                    } else {
                        anim.frames.len() - 1
                    };
                }
            }
        }
    }

    pub fn get_uv_coords(&self) -> Option<[[f32; 2]; 4]> {
        self.animations
            .get(&self.state)
            .and_then(|anim| anim.frames.get(self.current_frame))
            .map(|frame| frame.uv_coords)
    }
}
