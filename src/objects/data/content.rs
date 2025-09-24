// use crate::objects::{file::File, Directory};
use serde::{Deserialize, Serialize};

use crate::objects::data::{directory::Directory, file::File};

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
