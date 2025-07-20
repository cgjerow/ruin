use crate::{components_systems::Entity, world::World};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ActionState {
    Custom(u8),
}

#[derive(Debug, Clone)]
pub struct ActionStateComponent {
    pub state: ActionState,
}

impl From<u8> for ActionState {
    fn from(i: u8) -> Self {
        match i {
            other => ActionState::Custom(other),
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
