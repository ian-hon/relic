use std::{fs, path::Path};

use crate::core::{
    data::commit::Commit,
    error::{IOError, RelicError, SanctumError},
    object::Object,
    oid::ObjectID,
    util::string_to_oid,
};

fn fetch_from_commit_file(path: &Path, sanctum_path: &Path) -> Result<Option<Commit>, RelicError> {
    if !path.exists() {
        return Err(RelicError::IOError(IOError::FileNoExist));
    }

    if !sanctum_path.exists() {
        return Err(RelicError::SanctumError(SanctumError::SanctumNotFound));
    }

    if let Ok(oid_raw) = fs::read(path) {
        if oid_raw.is_empty() {
            return Ok(None);
        }

        let oid_raw: ObjectID = string_to_oid(str::from_utf8(&oid_raw).unwrap()).into();

        match oid_raw.construct(&sanctum_path) {
            Ok(c) => {
                return match c {
                    Object::Commit(c) => {
                        // println!("{}\n{}", c.get_oid().to_string(), c.serialise());
                        Ok(Some(c))
                    }
                    _ => Err(RelicError::ConfigurationIncorrect),
                };
            }
            Err(_) => return Err(RelicError::ConfigurationIncorrect),
        }
    }
    Err(RelicError::IOError(IOError::FileCantOpen))
}

pub fn fetch_head(relic_path: &Path) -> Result<Option<Commit>, RelicError> {
    fetch_from_commit_file(&relic_path.join("head"), &relic_path.join("sanctum"))
}

pub fn fetch_upstream(relic_path: &Path) -> Result<Option<Commit>, RelicError> {
    fetch_from_commit_file(&relic_path.join("upstream"), &relic_path.join("sanctum"))
}
