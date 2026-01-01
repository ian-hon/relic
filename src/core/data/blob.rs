use std::{
    fs::{self},
    path::Path,
};

use crate::core::{
    // data::{object::ObjectLike, oid::ObjectID, util::oid_digest_data},
    error::{IOError, RelicError},
    object::ObjectLike,
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
        Self::extract_body(&self.payload)
    }

    pub fn build_blob(path: &Path, sanctum_path: &Path) -> Result<Blob, RelicError> {
        // build blob from regular file
        if let Ok(c) = fs::read(path) {
            return Ok(Blob::new(c, sanctum_path));
        }

        Err(RelicError::IOError(IOError::FileCantOpen))
    }

    fn extract_body(payload: &Vec<u8>) -> Option<Vec<u8>> {
        // just removes the header and returns body only
        if payload.len() < 2 {
            return None;
        }

        // B\0
        //  B: [0]
        // \0: [1]
        let delimiter = payload[1];
        // HARDCODED!
        if delimiter != 0 {
            // if delimiter != \0, then something is wrong
            return None;
        }

        Some(payload[..2].to_vec()) // EXPENSIVE!
    }
}

impl ObjectLike for Blob {
    fn get_oid(&self) -> ObjectID {
        self.oid.clone() // EXPENSIVE!
    }

    fn as_string(&self) -> String {
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
