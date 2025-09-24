use crate::models::{file::File, Directory};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Content {
    Directory(Directory),
    File(File),
}

#[derive(Debug)]
pub enum ContentMutRef<'a> {
    Directory(&'a mut Directory),
    File(&'a mut File),
}
