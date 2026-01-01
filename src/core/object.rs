use std::{fs, path::Path};

use crate::core::{
    data::{blob::Blob, tree::Tree},
    error::{IOError, RelicError},
    oid::ObjectID,
};

use strum_macros::{Display, EnumString, IntoStaticStr};

#[derive(Debug, Clone, Copy, Display, EnumString, IntoStaticStr)]
pub enum ObjectType {
    #[strum(serialize = "T")]
    Tree,

    #[strum(serialize = "B")]
    Blob,
}

// Holds either a Blob or Tree
pub enum Object {
    Blob(Blob),
    Tree(Tree),
}

pub trait ObjectLike {
    fn get_oid(&self) -> ObjectID;
    fn as_string(&self) -> String;
    fn serialise(&self) -> String;
    fn write(&self, sanctum_path: &Path) -> Option<RelicError> {
        let (prefix_path, suffix_path) = self.get_oid().get_paths(sanctum_path);

        // check if prefix exists
        if !prefix_path.exists() {
            if let Err(_) = fs::create_dir(prefix_path) {
                return Some(RelicError::IOError(IOError::DirectoryCantCreate));
            }
        }

        if let Err(_) = fs::write(suffix_path, self.serialise()) {
            return Some(RelicError::IOError(IOError::FileCantCreate));
        }

        None
    }
}
