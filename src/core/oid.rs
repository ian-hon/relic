use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::core::{
    data::{blob::Blob, commit::Commit, tree::Tree},
    error::{IOError, RelicError, SanctumError},
    object::{Object, ObjectType},
    util::{empty_oid, oid_to_string, string_to_oid},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectID(pub [u8; 32]);
impl ObjectID {
    pub fn new(oid: [u8; 32]) -> ObjectID {
        ObjectID(oid)
    }

    pub fn from_string(content: &str) -> ObjectID {
        ObjectID(string_to_oid(content))
    }

    pub fn get_segments(&self) -> (String, String) {
        // returns prefix and suffix
        // abcdef12345 -> ('ab', 'cdef12345')
        (
            self.to_string()[..2].to_string(),
            self.to_string()[2..].to_string(),
        )
    }

    pub fn get_paths(&self, sanctum_path: &Path) -> (PathBuf, PathBuf) {
        let (prefix, suffix) = self.get_segments();

        let prefix_path = sanctum_path.join(&prefix);
        let suffix_path = prefix_path.join(&suffix);
        (prefix_path, suffix_path)
    }

    pub fn empty() -> ObjectID {
        ObjectID(empty_oid())
    }

    pub fn construct(&self, sanctum_path: &Path) -> Result<Object, RelicError> {
        // converts from OID to actual object, fetching from sanctum
        if !sanctum_path.exists() {
            return Err(RelicError::SanctumError(SanctumError::SanctumNotFound));
        }

        let (prefix_path, suffix_path) = self.get_paths(sanctum_path);

        // println!("{prefix_path:?}\t{suffix_path:?}");

        // check if prefix & suffix exists
        if prefix_path.exists() && suffix_path.exists() {
            // TODO: infer ObjectType from header

            if let Ok(payload) = fs::read(suffix_path) {
                if let Some(t) = Object::extract_header(&payload) {
                    match t {
                        ObjectType::Blob => {
                            let b = Blob::deserialise(payload);

                            // println!("Blob\n\n{:?}", b.serialise());

                            return Ok(Object::Blob(b));
                        }
                        ObjectType::Tree => {
                            if let Some(t) = Tree::deserialise(payload) {
                                // println!("Tree\n\n{:?}", t.serialise());

                                return Ok(Object::Tree(t));
                            }
                            // println!("deserialise failed");
                            return Err(RelicError::ConfigurationIncorrect);
                        }
                        ObjectType::Commit => {
                            if let Some(c) = Commit::deserialise(payload) {
                                // println!("Commit\n\n{:?}", c.serialise());

                                return Ok(Object::Commit(c));
                            }

                            // println!("commit deserialise failed");
                            return Err(RelicError::ConfigurationIncorrect);
                        }
                    }
                }
                return Err(RelicError::ConfigurationIncorrect);
            }
            // match otype {
            //     ObjectType::Tree => {
            //         if let Ok(content) = fs::read_to_string(suffix_path) {
            //             if let Some(t) = Tree::from_string(&content) {
            //                 return Ok(Object::Tree(t));
            //             }
            //             return Err(RelicError::ConfigurationIncorrect);
            //         }
            //     }
            //     ObjectType::Blob => {
            //         if let Ok(c) = fs::read(suffix_path) {
            //             return Ok(Object::Blob(Blob {
            //                 oid: self.clone(),
            //                 length: c.len(),
            //                 content: c,
            //             }));
            //         }
            //     }
            // }

            return Err(RelicError::IOError(IOError::FileCantOpen));
        }

        Err(RelicError::SanctumError(SanctumError::RecordNoExist))
    }

    pub fn as_trunc(&self) -> String {
        (&self.to_string()[..6]).to_string().to_owned()
    }
}

impl Into<ObjectID> for [u8; 32] {
    fn into(self) -> ObjectID {
        ObjectID(self)
    }
}

impl ToString for ObjectID {
    fn to_string(&self) -> String {
        oid_to_string(self.0)
    }
}
