#[derive(Debug, Clone, Copy, Default)]
pub struct HitboxComponent {
    pub offset: [f32; 2], // relative to entity's position
    pub size: [f32; 2],   // width and height
    pub active: bool,     // toggle on/off
}
