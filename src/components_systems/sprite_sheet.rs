use crate::texture::Texture;

#[derive(Clone, Debug)]
pub struct SpriteSheetComponent {
    pub texture_id: String,
    pub texture: Texture,
}
