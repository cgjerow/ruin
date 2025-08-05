use ruin_assets::{Handle, ImageTexture};

#[derive(Clone, Debug)]
pub struct SpriteSheetComponent {
    pub image_texture: Handle<ImageTexture>,
}
