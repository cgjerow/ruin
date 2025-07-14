use std::collections::HashMap;

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

#[derive(Clone, Debug)]
pub struct SpriteSheetComponent {
    pub texture_id: String,
    pub texture: Texture,
}

#[derive(Debug, Clone)]
pub struct TransformComponent {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub acceleration: [f32; 3],
    pub size: [f32; 2],
}

pub fn transform_system_physics(world: &mut World, delta_time: f32) {
    for transform in world.transforms.values_mut() {
        // Integrate acceleration into velocity
        transform.velocity[0] += transform.acceleration[0] * delta_time;
        transform.velocity[1] += transform.acceleration[1] * delta_time;

        // Apply velocity to position
        transform.position[0] += transform.velocity[0] * delta_time;
        transform.position[1] += transform.velocity[1] * delta_time;

        // Apply drag or friction (optional)
        let drag = 0.8;
        transform.velocity[0] *= drag;
        transform.velocity[1] *= drag;

        // Clear acceleration after use
        transform.acceleration = [0.0; 3];
    }
}

pub fn transform_system_calculate_intended_position(
    world: &World,
    delta_time: f32,
) -> HashMap<Entity, TransformComponent> {
    let mut to_return = HashMap::new();
    for (id, before) in world.transforms.iter() {
        let mut transform = before.clone();
        // Integrate acceleration into velocity
        transform.velocity[0] += transform.acceleration[0] * delta_time;
        transform.velocity[1] += transform.acceleration[1] * delta_time;

        // Clamp speed (optional, example max: 300.0)
        let speed = (transform.velocity[0].powi(2) + transform.velocity[1].powi(2)).sqrt();
        let max_speed = 30.0;
        if speed > max_speed {
            let scale = max_speed / speed;
            transform.velocity[0] *= scale;
            transform.velocity[1] *= scale;
        }

        // Apply velocity to position
        transform.position[0] += transform.velocity[0] * delta_time;
        transform.position[1] += transform.velocity[1] * delta_time;

        // Apply drag or friction (optional)
        let drag = 0.8;
        transform.velocity[0] *= drag;
        transform.velocity[1] *= drag;

        // Clear acceleration after use
        transform.acceleration = [0.0; 3];

        to_return.insert(id.clone(), transform);
    }
    to_return
}

pub fn transform_system_add_acceleration(world: &mut World, id: Entity, dx: f32, dy: f32) {
    if let Some(transform) = world.transforms.get_mut(&id) {
        transform.acceleration[0] += dx;
        transform.acceleration[1] += dy;
    }
}

pub fn transform_system_redirect(
    world: &mut World,
    id: Entity,
    dx: f32,
    dy: f32,
    sep_x: f32,
    sep_y: f32,
) {
    if let Some(transform) = world.transforms.get_mut(&id) {
        // Apply bounce velocity
        transform.velocity[0] = dx;
        transform.velocity[1] = dy;

        // Clear current acceleration
        transform.acceleration[0] = 0.0;
        transform.acceleration[1] = 0.0;

        // Apply small separation offset
        transform.position[0] += sep_x;
        transform.position[1] += sep_y;
    }
}

impl Animation {
    pub fn from_lua_table(table: mlua::Table) -> (Self, String) {
        let looped: bool = table.get("looped").unwrap_or(true);

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
            },
            sprite_path,
        )
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

pub fn set_entity_state(world: &mut World, entity: Entity, state: ActionState) {
    if let Some(current) = world.action_states.get(&entity) {
        if current.state == state {
            return;
        }
    }

    world.action_states.insert(
        entity,
        ActionStateComponent {
            state: state.clone(),
        },
    );

    if let Some(animations) = world.animations.get_mut(&entity) {
        if let Some(anim) = animations.animations.get_mut(&state) {
            animations.current_frame_index = 0;
            animations.current_frame = anim.frames[0].clone();
            animations.frame_timer = 0.0;
        };
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FlipComponent {
    pub x: bool, // flip horizontally
    pub y: bool, // flip vertically
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HitboxComponent {
    pub offset: [f32; 3], // relative to entity's position
    pub size: [f32; 3],   // width and height
    pub active: bool,     // toggle on/off
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ColliderComponent {
    pub offset: [f32; 3],
    pub size: [f32; 3],
    pub is_solid: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct CollisionInfo {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub next_pos_a: [f32; 3],
    pub next_pos_b: [f32; 3],
    pub velocity_a: [f32; 3],
    pub velocity_b: [f32; 3],
    pub normal: [f32; 3],
}

pub fn collision_system(
    world: &World,
    next_transforms: &HashMap<Entity, TransformComponent>,
) -> Vec<CollisionInfo> {
    let mut collisions = Vec::new();

    for (entity, next_transform) in next_transforms.iter() {
        if let Some(collider) = world.colliders.get(entity) {
            for (other_entity, other_collider) in world.colliders.iter() {
                if entity == other_entity {
                    continue;
                }

                if let Some(other_next_transform) = next_transforms.get(other_entity) {
                    if is_colliding(
                        next_transform,
                        collider,
                        other_next_transform,
                        other_collider,
                    ) {
                        // Compute collision normal from A to B
                        let dx = other_next_transform.position[0] - next_transform.position[0];
                        let dy = other_next_transform.position[1] - next_transform.position[1];
                        let dz = other_next_transform.position[2] - next_transform.position[2];
                        let mag = (dx * dx + dy * dy + dz * dz).sqrt();

                        let normal = if mag != 0.0 {
                            [dx / mag, dy / mag, dz / mag]
                        } else {
                            [0.0, 0.0, 0.0] // exact overlap
                        };

                        collisions.push(CollisionInfo {
                            entity_a: *entity,
                            entity_b: *other_entity,
                            next_pos_a: next_transform.position,
                            next_pos_b: other_next_transform.position,
                            velocity_a: next_transform.velocity,
                            velocity_b: other_next_transform.velocity,
                            normal,
                        });
                    }
                }
            }
        }
    }

    collisions
}

pub fn is_colliding(
    a_transform: &TransformComponent,
    a_collider: &ColliderComponent,
    b_transform: &TransformComponent,
    b_collider: &ColliderComponent,
) -> bool {
    // Assuming position is center of entity and size is width/height/depth
    let a_min = [
        a_transform.position[0] - a_collider.size[1] / 2.0,
        a_transform.position[1] - a_collider.size[1] / 2.0,
        a_transform.position[2] - a_collider.size[2] / 2.0,
    ];
    let a_max = [
        a_transform.position[0] + a_collider.size[0] / 2.0,
        a_transform.position[1] + a_collider.size[1] / 2.0,
        a_transform.position[2] + a_collider.size[2] / 2.0,
    ];

    let b_min = [
        b_transform.position[0] - b_collider.size[0] / 2.0,
        b_transform.position[1] - b_collider.size[1] / 2.0,
        b_transform.position[2] - b_collider.size[2] / 2.0,
    ];
    let b_max = [
        b_transform.position[0] + b_collider.size[0] / 2.0,
        b_transform.position[1] + b_collider.size[1] / 2.0,
        b_transform.position[2] + b_collider.size[2] / 2.0,
    ];

    // Check for overlap on all 3 axes (X, Y, Z)
    let overlap_x = a_min[0] <= b_max[0] && a_max[0] >= b_min[0];
    let overlap_y = a_min[1] <= b_max[1] && a_max[1] >= b_min[1];
    let overlap_z = a_min[2] <= b_max[2] && a_max[2] >= b_min[2];

    overlap_x && overlap_y && overlap_z
}
