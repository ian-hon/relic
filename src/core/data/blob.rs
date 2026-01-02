use std::{
    fs::{self},
    path::Path,
};

use crate::core::{
    // data::{object::ObjectLike, oid::ObjectID, util::oid_digest_data},
    error::{IOError, RelicError},
    object::{Object, ObjectLike},
    oid::ObjectID,
    util::oid_digest_data,
};

/*
Blob format:
B\0
{actual content}
*/

const HEADER: &str = "B\0";

#[derive(Debug)]
pub struct Blob {
    oid: ObjectID,
    pub payload: Vec<u8>,
}
impl Blob {
    pub fn new(mut raw: Vec<u8>, sanctum_path: &Path) -> Blob {
        // raw does NOT contain header
        // raw contains ONLY the raw data

        // TODO: test
        let mut payload = HEADER.as_bytes()[..].to_vec();
        payload.append(&mut raw);

        let b = Blob {
            oid: ObjectID::new(oid_digest_data(&payload)),
            payload,
        };

        b.write(sanctum_path);

        b
    }

    pub fn deserialise(payload: Vec<u8>) -> Blob {
        // payload includes header
        // just directly compute oid, dont need to write to sanctum for this

        Blob {
            oid: ObjectID::new(oid_digest_data(&payload)),
            payload,
        }
    }

    pub fn deserialise_and_write(
        payload: Vec<u8>,
        sanctum_path: &Path,
    ) -> Result<Blob, RelicError> {
        let b = Blob {
            oid: ObjectID::new(oid_digest_data(&payload)),
            payload,
        };

        match b.write(sanctum_path) {
            Some(e) => Err(e),
            None => Ok(b),
        }
    }

    pub fn get_body(&self) -> Option<Vec<u8>> {
        // just removes the header and returns body only
        // TODO: FIX
        // get_body used only in as_string
        // as_string not used anywhere
        Object::extract_body(&self.payload)
    }

    pub fn build_blob(path: &Path, sanctum_path: &Path) -> Result<Blob, RelicError> {
        // build blob from regular file
        if let Ok(c) = fs::read(path) {
            return Ok(Blob::new(c, sanctum_path));
        }

        Err(RelicError::IOError(IOError::FileCantOpen))
    }
}

impl ObjectLike for Blob {
    fn get_oid(&self) -> ObjectID {
        self.oid.clone() // EXPENSIVE!
    }

    fn as_string(&self) -> String {
        // BROKEN
        // returns without header

        // TODO: handle invalid utf-8s
        if let Some(body) = self.get_body() {
            return str::from_utf8(&body).map_or("".to_string(), |s| s.to_string());
        }
        "".to_string()
    }

    fn serialise(&self) -> String {
        // returns with header

        // TODO: handle invalid utf-8s
        str::from_utf8(&self.payload).map_or("".to_string(), |s| s.to_string())
    }
}
