#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entity(pub u32);

impl From<Entity> for u32 {
    fn from(e: Entity) -> Self {
        e.0
    }
}
