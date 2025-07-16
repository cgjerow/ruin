use crate::{components_systems::Entity, world::World};

#[derive(Debug, Clone)]
pub struct HealthComponent {
    pub total: u16,
    pub current: u16,
}

pub fn damage(world: &mut World, entity: &Entity, amount: u16) -> bool {
    if let Some(health) = world.health_bars.get_mut(entity) {
        health.current = health.current.saturating_sub(amount);
        if health.current <= 0 {
            return true;
        }
    }
    false
}
