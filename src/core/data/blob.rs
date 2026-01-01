use std::{
    fs::{self},
    path::Path,
};

use crate::core::{
    data::{object::ObjectLike, oid::ObjectID, util::oid_digest_data},
    error::RelicError,
};

/*
Blob format:
B\0
{actual content}
*/

pub struct Blob {
    pub oid: ObjectID,
    pub length: usize,
    pub content: Vec<u8>,
}
impl Blob {
    pub fn new(raw: Vec<u8>, sanctum_path: &Path) -> Blob {
        let b = Blob {
            oid: ObjectID::new(oid_digest_data(&raw)),
            length: raw.len(),
            content: raw,
        };

        b.write(sanctum_path);

        b
    }

    pub fn load_from_path(path: &Path, sanctum_path: &Path) -> Result<Blob, RelicError> {
        if let Ok(c) = fs::read(path) {
            return Ok(Blob::new(c, sanctum_path));
        }

        Err(RelicError::FileCantOpen)
    }

    pub fn as_string(&self) -> Option<String> {
        str::from_utf8(&self.content).map_or(None, |s| Some(s.to_string())) // EXPENSIVE!
    }
}

impl ObjectLike for Blob {
    fn get_oid(&self) -> ObjectID {
        self.oid.clone() // EXPENSIVE!
    }

    fn as_string(&self) -> String {
        // TODO: handle invalid utf-8s
        self.as_string().unwrap_or_else(|| "".to_string())
    }
}
