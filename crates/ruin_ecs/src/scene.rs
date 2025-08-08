use crate::AnimationComponent;

pub struct Scene {
    pub elements: Vec<Element>,
}

pub struct Element {
    pub animation: AnimationComponent,
    pub elements: Vec<Element>,
}
