use crate::AnimationComponent;

pub struct Scene {
    pub elements: Vec<Element>,
    pub scenes: Vec<Scene>,
}

pub struct Element {
    pub animation: AnimationComponent,
}
