use std::{
    fs::{self, File},
    path::Path,
};

use sha2::{Digest, Sha256};

use crate::core::{
    data::{
        object::Object,
        oid::ObjectID,
        util::{oid_digest, oid_digest_data},
    },
    error::RelicError,
};

/*
Blob format:
header not necessary
{actual content}
*/

pub struct Blob {
    pub oid: ObjectID,
    length: usize,
    pub content: Vec<u8>,
}
impl Blob {
    pub fn new(raw: Vec<u8>) -> Blob {
        Blob {
            oid: ObjectID::new(oid_digest_data(&raw)),
            length: raw.len(),
            content: raw,
        }
    }

    pub fn load_from_path(path: &Path) -> Result<Blob, RelicError> {
        if let Ok(c) = fs::read(path) {
            return Ok(Blob::new(c));
        }

        Err(RelicError::FileCantOpen)
    }

    pub fn as_string(&self) -> Option<String> {
        str::from_utf8(&self.content).map_or(None, |s| Some(s.to_string())) // EXPENSIVE!
    }
}

impl Object for Blob {
    fn get_oid(&self) -> ObjectID {
        self.oid.clone() // EXPENSIVE!
    }

    fn as_string(&self) -> String {
        self.as_string().unwrap_or_else(|| "".to_string())
    }
}
