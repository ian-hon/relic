use std::{fs, path::PathBuf};

use crate::core::{
    data::commit::Commit,
    error::{IOError, RelicError},
    object::Object,
    oid::ObjectID,
    tracking::content_set::ContentSet,
    util::string_to_oid,
};

pub struct State {
    pub root_path: PathBuf,
    pub relic_path: PathBuf,

    pub tracking_set: ContentSet,
    pub ignore_set: ContentSet,
}
impl State {
    pub fn construct(root_path: PathBuf) -> Option<State> {
        // load tracking and ignore set
        let relic_path = root_path.join(".relic");
        if !relic_path.exists() {
            return None;
        }

        let tracking_set = ContentSet::construct(&relic_path.join("tracked")).ok()?;
        let ignore_set = ContentSet::construct(&root_path.join(".relic_ignore")).ok()?;

        Some(State {
            root_path,
            relic_path,
            tracking_set,
            ignore_set,
        })
    }

    fn fetch_from_commit_file(&self, path: PathBuf) -> Result<Option<Commit>, RelicError> {
        if !path.exists() {
            return Err(RelicError::IOError(IOError::FileNoExist));
        }

        if let Ok(oid_raw) = fs::read(path) {
            if oid_raw.is_empty() {
                return Ok(None);
            }

            let oid_raw: ObjectID = string_to_oid(str::from_utf8(&oid_raw).unwrap()).into();

            match oid_raw.construct(&self.get_sanctum_path()) {
                Ok(c) => {
                    return match c {
                        Object::Commit(c) => Ok(Some(c)),
                        _ => Err(RelicError::ConfigurationIncorrect),
                    };
                }
                Err(_) => return Err(RelicError::ConfigurationIncorrect),
            }
        }
        Err(RelicError::IOError(IOError::FileCantOpen))
    }

    pub fn fetch_head(&self) -> Result<Option<Commit>, RelicError> {
        self.fetch_from_commit_file(self.get_head_path())
    }

    pub fn fetch_upstream(&self) -> Result<Option<Commit>, RelicError> {
        self.fetch_from_commit_file(self.get_upstream_path())
    }

    pub fn update_tracking_set(&self) {
        let _ = fs::write(
            self.relic_path.join("tracked"),
            self.tracking_set.serialise(),
        );
    }

    // #region paths
    pub fn get_sanctum_path(&self) -> PathBuf {
        let s = self.relic_path.join("sanctum");
        if !s.exists() {
            // TODO: handle exceptions
            fs::create_dir(&s).unwrap();
        }
        s
    }

    pub fn get_head_path(&self) -> PathBuf {
        self.relic_path.join("head")
    }

    pub fn get_upstream_path(&self) -> PathBuf {
        self.relic_path.join("upstream")
    }
    // #endregion
}
