use sha2::{Digest, Sha256};

use crate::core::data::{object::Object, oid::ObjectID, util::oid_digest};

/*
Blob format:
header not necessary
{actual content}
*/

pub struct Blob {
    pub oid: ObjectID,
    length: usize,
    pub content: String,
}
impl Blob {
    pub fn new(raw: &str) -> Blob {
        Blob {
            oid: ObjectID::new(oid_digest(raw)),
            length: raw.len(),
            content: raw.to_string(), // EXPENSIVE!
        }
    }

    pub fn as_string(&self) -> String {
        self.content.clone() // EXPENSIVE!
    }
}

impl Object for Blob {
    fn get_oid(&self) -> ObjectID {
        self.oid.clone() // EXPENSIVE!
    }

    fn as_string(&self) -> String {
        self.as_string()
    }
}
