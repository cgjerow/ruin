use std::fmt;
use std::hash::{Hash, Hasher};

// Static and Thrad-Safe
pub trait Asset: 'static + Send + Sync {}

#[derive(Clone, Eq)]
pub struct AssetPath {
    path: String,
}

impl AssetPath {
    pub fn new<S: Into<String>>(path: S) -> Self {
        Self { path: path.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.path
    }
}

// Implement Debug for easier debugging
impl fmt::Debug for AssetPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AssetPath").field(&self.path).finish()
    }
}

// Implement PartialEq to compare by path string
impl PartialEq for AssetPath {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

// Implement Hash to hash the path string
impl Hash for AssetPath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}
