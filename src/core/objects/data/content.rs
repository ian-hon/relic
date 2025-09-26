use serde::{Deserialize, Serialize};

use crate::core::{Blob, Tree};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Content {
    Directory(Tree),
    Blob(Blob),
}

#[derive(Debug)]
pub enum ContentMutRef<'a> {
    Directory(&'a mut Tree),
    Blob(&'a mut Blob),
}
