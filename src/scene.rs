use std::collections::HashMap;

use crate::{components_systems::AnimationComponent, graphics_2d::TextureId};

pub struct Scene {
    pub elements: Vec<Element>,
    pub scenes: Vec<Scene>,
}

pub struct Element {
    pub animation: AnimationComponent,
}
