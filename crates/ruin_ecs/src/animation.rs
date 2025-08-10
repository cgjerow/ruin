use cgmath::Vector2;
use ruin_assets::{Handle, ImageTexture};
use ruin_bitmaps::vecbool_to_u8;
use std::collections::HashMap;

use crate::{
    physics_2d::{Area2D, HalfExtents, Shape2D},
    world::World,
    ActionState,
};

#[derive(Debug, Clone)]
pub struct SpriteFrame {
    pub shape: Shape2D,
    pub uv_coords: [[f32; 2]; 4], // bottom-left, bottom-right, top-right, top-left
    pub duration: f32,
    pub hitboxes: Vec<Area2D>,
    pub hurtboxes: Vec<Area2D>,
    pub frame_pixel_dims: [f32; 2],
}

#[derive(Debug, Clone)]
pub struct Animation {
    // this should change to TextureId
    pub sprite_sheet_id: Handle<ImageTexture>,
    pub is_transparent: bool,
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
    pub fn raw_from_lua_table(
        table: mlua::Table,
        texture_loader: &mut impl FnMut(String) -> Handle<ImageTexture>,
    ) -> Self {
        let looped: bool = table.get("looped").unwrap_or(true);
        let is_transparent: bool = table.get("is_transparent").unwrap_or(false);

        let frames_table: mlua::Table = table
            .get("frames")
            .expect("Missing 'frames' table in animation");

        println!("SPRITE");
        let sprite_path: String = table
            .get("sprite")
            .expect("Sprite Sheet is required for animation.");
        println!("SPRITE {:?}", sprite_path);

        //let tex_w: f32 = table.get("tex_width").unwrap();
        //let tex_h: f32 = table.get("tex_height").unwrap();
        let tex_w = 384.0;
        let tex_h = 32.0;

        let mut frames = Vec::new();
        for (i, pair) in frames_table.sequence_values::<mlua::Table>().enumerate() {
            if let Ok(frame_data) = pair {
                let x: f32 = frame_data.get("x").unwrap();
                let y: f32 = frame_data.get("y").unwrap();
                let w: f32 = frame_data.get("w").unwrap_or(1.0);
                let h: f32 = frame_data.get("h").unwrap_or(1.0);

                let duration: f32 = frame_data.get("duration").unwrap_or(1.0);

                let u0 = x / tex_w;
                let u1 = (x + w) / tex_w;
                let v1 = 1.0 - (y / tex_h);
                let v0 = 1.0 - ((y + h) / tex_h);

                // WGPU uses origin at top-left by default. Flip Y if needed.
                let uv_coords = [[u0, v1], [u1, v1], [u1, v0], [u0, v0]];

                // Lets base shape off of half extents for now
                let shape = Shape2D::Rectangle {
                    half_extents: HalfExtents { x: w, y: h },
                };

                let frame_pixel_dims = [16.0, 16.0]; // I really don't know why I need this right
                                                     // now
                frames.push(SpriteFrame {
                    shape,
                    uv_coords,
                    duration,
                    hitboxes: Vec::new(),
                    hurtboxes: Vec::new(),
                    frame_pixel_dims,
                });
            }
        }

        Animation {
            sprite_sheet_id: texture_loader(sprite_path),
            frames,
            looped,
            is_transparent,
        }
    }

    pub fn from_lua_table(
        table: mlua::Table,
        texture_loader: &mut impl FnMut(String) -> Handle<ImageTexture>,
    ) -> Self {
        let looped: bool = table.get("looped").unwrap_or(true);
        let is_transparent: bool = table.get("is_transparent").unwrap_or(false);

        let frames_table: mlua::Table = table
            .get("frames")
            .expect("Missing 'frames' table in animation");
        let hitboxes: mlua::Table = table.get("hitboxes").unwrap();
        let hurtboxes: mlua::Table = table.get("hurtboxes").unwrap();

        let tex_w: f32 = table.get("sprite_sheet_width").unwrap();
        let tex_h: f32 = table.get("sprite_sheet_height").unwrap();
        let tile_w: f32 = table.get("tile_width").unwrap();
        let tile_h: f32 = table.get("tile_height").unwrap();

        let sprite_path: String = table
            .get("sprite")
            .expect("Sprite Sheet is required for animation.");

        let mut frames = Vec::new();
        for (i, pair) in frames_table.sequence_values::<mlua::Table>().enumerate() {
            if let Ok(frame_data) = pair {
                let x: f32 = frame_data.get("x").unwrap();
                let y: f32 = frame_data.get("y").unwrap();
                // h,w are currently not really being set from lua
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
                // Lets base shape off of half extents for now
                let shape = Shape2D::Rectangle {
                    half_extents: HalfExtents { x: w, y: h },
                };

                let frame_pixel_dims = [tile_w, tile_h];
                frames.push(SpriteFrame {
                    shape,
                    uv_coords,
                    duration,
                    hitboxes: parse_hitboxes_from_table(&hitboxes, i, frame_pixel_dims),
                    hurtboxes: parse_hitboxes_from_table(&hurtboxes, i, frame_pixel_dims),
                    frame_pixel_dims,
                });
            }
        }

        Animation {
            sprite_sheet_id: texture_loader(sprite_path),
            frames,
            looped,
            is_transparent,
        }
    }
}

fn parse_hitboxes_from_table(
    table: &mlua::Table,
    index: usize,
    frame_size: [f32; 2],
) -> Vec<Area2D> {
    let mut boxes = Vec::new();
    let frame_boxes: mlua::Table = match table.get((index + 1) as i64) {
        Ok(t) => t,
        Err(_) => return boxes,
    };

    for entry in frame_boxes.sequence_values::<mlua::Table>() {
        if let Ok(b) = entry {
            let x: f32 = b.get("center_x").unwrap_or(0.0);
            let y: f32 = b.get("center_y").unwrap_or(0.0);
            let w: f32 = b.get("width").unwrap_or(0.0);
            let h: f32 = b.get("height").unwrap_or(0.0);
            let layers: [bool; 8] = b.get("layers").unwrap_or_default();
            let masks: [bool; 8] = b.get("masks").unwrap_or_default();

            let frame_center_x = frame_size[0] * 0.5;
            let frame_center_y = frame_size[1] * 0.5;

            let offset_x = x - frame_center_x;
            let offset_y = y - frame_center_y;

            boxes.push(Area2D {
                shape: Shape2D::Rectangle {
                    half_extents: Vector2::new(w * 0.5, h * 0.5),
                },
                offset: Vector2::new(offset_x, offset_y),
                active: true,
                layers: vecbool_to_u8(layers),
                masks: vecbool_to_u8(masks),
            });
        }
    }

    boxes
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
