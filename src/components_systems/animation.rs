use std::collections::HashMap;

use crate::{
    components_systems::{ActionState, Entity},
    world::World,
};

#[derive(Debug, Clone)]
pub struct SpriteFrame {
    pub uv_coords: [[f32; 2]; 4], // bottom-left, bottom-right, top-right, top-left
    pub duration: f32,
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub is_transparent: bool,
    pub sprite_sheet_id: Entity,
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

impl Animation {
    pub fn from_lua_table(table: mlua::Table) -> (Self, String) {
        let looped: bool = table.get("looped").unwrap_or(true);
        let is_transparent: bool = table.get("is_transparent").unwrap_or(false);

        let frames_table: mlua::Table = table
            .get("frames")
            .expect("Missing 'frames' table in animation");

        let tex_w = table.get("sprite_sheet_width").unwrap_or(1.0);
        let tex_h = table.get("sprite_sheet_height").unwrap_or(1.0);
        let sprite_path: String = table
            .get("sprite")
            .expect("Sprite Sheet is required for animation.");

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
        (
            Animation {
                sprite_sheet_id: Entity(0),
                frames,
                looped,
                is_transparent,
            },
            sprite_path,
        )
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
