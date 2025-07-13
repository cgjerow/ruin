use std::{collections::HashMap, u16};

use crate::texture::Texture;
use crate::world::World;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entity(pub u32);
impl From<Entity> for u32 {
    fn from(e: Entity) -> Self {
        e.0
    }
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

#[derive(Debug, Clone)]
pub struct AnimationComponent {
    pub animations: HashMap<ActionState, Animation>,
    pub current_frame: SpriteFrame,
    pub current_frame_index: usize,
    pub frame_timer: f32,
}

#[derive(Clone, Debug)]
pub struct SpriteSheetComponent {
    pub texture_id: String,
    pub texture: Texture,
}

#[derive(Debug, Clone)]
pub struct TransformComponent {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub size: [f32; 2],
}

pub fn transform_system_calculate_position(world: &mut World, delta_time: f32) {
    for (_id, transform) in world.transforms.iter_mut() {
        for i in 0..3 {
            transform.position[i] += transform.velocity[i] * delta_time;
            transform.velocity[i] = 0.0; // zero out velocity after applying
        }
    }
}
pub fn transform_system_add_velocity(world: &mut World, id: Entity, dx: f32, dy: f32) {
    if let Some(transform) = world.transforms.get_mut(&id) {
        transform.velocity[0] += dx;
        transform.velocity[1] += dy;
    }
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
                let u1 = (x + w) / tex_w;
                let v1 = 1.0 - (y / tex_h);
                let v0 = 1.0 - ((y + h) / tex_h);

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
pub enum ActionState {
    Idle,
    Walking,
    Running,
    Jumping,
    Landing,
    Dying,
    Colliding,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct ActionStateComponent {
    pub state: ActionState,
}

impl From<String> for ActionState {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Idle" => ActionState::Idle,
            "Walking" => ActionState::Walking,
            "Running" => ActionState::Running,
            "Jumping" => ActionState::Jumping,
            "Landing" => ActionState::Landing,
            "Dying" => ActionState::Dying,
            "Colliding" => ActionState::Colliding,
            other => ActionState::Custom(other.to_string()),
        }
    }
}

pub fn animation_system_update_frames(world: &mut World, dt: f32) {
    for (entity, animation) in world.animations.iter_mut() {
        if let Some(action_state) = world.action_states.get(entity) {
            if let Some(anim) = animation.animations.get(&action_state.state) {
                if anim.frames.is_empty() {
                    return;
                }

                animation.frame_timer += dt;

                let current = animation.current_frame_index.min(anim.frames.len() - 1);
                let frame_duration = anim.frames[current].duration;

                if animation.frame_timer >= frame_duration {
                    animation.frame_timer -= frame_duration; // carry over extra time
                    animation.current_frame_index += 1;
                    if animation.current_frame_index >= anim.frames.len() {
                        animation.current_frame_index = if anim.looped {
                            0
                        } else {
                            anim.frames.len() - 1
                        };
                    }
                    animation.current_frame = animation
                        .animations
                        .get(&action_state.state)
                        .unwrap()
                        .frames[animation.current_frame_index]
                        .clone();
                }
            }
        }
    }
}
