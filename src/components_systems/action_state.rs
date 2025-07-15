use crate::{components_systems::Entity, world::World};

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
