use serde::{Deserialize, Serialize};

use crate::core::{Blob, Tree};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Content {
    Tree(Tree),
    Blob(Blob),
}
impl Content {
    pub fn get_name(&self) -> String {
        match self {
            Content::Tree(tree) => tree.name.clone(),
            Content::Blob(blob) => blob.name.clone(),
        }
    }
}

#[derive(Debug)]
pub enum ContentMutRef<'a> {
    Tree(&'a mut Tree),
    Blob(&'a mut Blob),
}
