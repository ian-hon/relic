use std::{fs, path::Path, str::FromStr};

use crate::core::{
    data::{blob::Blob, commit::Commit, tree::Tree},
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

    #[strum(serialize = "C")]
    Commit,
}
impl ObjectType {
    pub fn from_u8(b: u8) -> Option<ObjectType> {
        // EXPENSIVE!
        str::from_utf8(&[b]).map_or_else(
            |_| None,
            |t| ObjectType::from_str(t).map_or_else(|_| None, |t| Some(t)),
        )
    }

    pub fn to_u8(&self) -> u8 {
        // EXPENSIVE!
        self.to_string().as_bytes()[0]
    }
}

// Holds either a Blob or Tree
#[derive(Debug)]
pub enum Object {
    Blob(Blob),
    Tree(Tree),
    Commit(Commit),
}
impl Object {
    pub fn extract_header(payload: &Vec<u8>) -> Option<ObjectType> {
        if payload.len() < 2 {
            return None;
        }

        ObjectType::from_u8(payload[0])
    }

    pub fn extract_body(payload: &Vec<u8>) -> Option<Vec<u8>> {
        // just removes the header and returns body only
        if payload.len() < 2 {
            return None;
        }

        // // B\0
        // //  B: [0]
        // // \0: [1]
        // let delimiter = payload[1];
        // // HARDCODED!
        // if delimiter != 0 {
        //     // if delimiter != \0, then something is wrong
        //     return None;
        // }

        Some(payload[2..].to_vec()) // EXPENSIVE!
    }
}

pub trait ObjectLike {
    fn get_oid(&self) -> ObjectID;
    #[allow(unused)]
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
