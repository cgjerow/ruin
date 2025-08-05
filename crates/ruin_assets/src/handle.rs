use std::{hash::Hash, hash::Hasher, marker::PhantomData};

use crate::Asset;

pub type Index = u32;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct HandleId {
    index: Index,
}

#[derive(Debug)]
pub struct Handle<T: Asset> {
    id: HandleId,
    marker: PhantomData<T>,
}

impl<T: Asset> Handle<T> {
    pub fn new(index: Index) -> Self {
        Self {
            id: HandleId { index },
            marker: PhantomData,
        }
    }

    pub fn id(&self) -> HandleId {
        self.id
    }
}

impl<A: Asset> Hash for Handle<A> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl<A: Asset> PartialEq for Handle<A> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<A: Asset> Eq for Handle<A> {}
impl<T: Asset> Copy for Handle<T> {}
impl<T: Asset> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}
