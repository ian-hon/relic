use std::{fs, path::PathBuf};

use crate::core::{
    branch::branch::{Branch, BranchSource},
    data::commit::Commit,
    error::{IOError, RelicError},
    object::Object,
    oid::ObjectID,
    tracking::content_set::ContentSet,
};

/* File structure:
.relic
    - branches/
        - local/
        - upstream/
    - sanctum/
    - head
    - tracked
*/

const RELIC_PATH: &str = ".relic";
const SANCTUM_PATH: &str = "sanctum";
const BRANCHES_PATH: &str = "branches";
const LOCAL_BRANCHES_PATH: &str = "local";
const UPSTREAM_BRANCHES_PATH: &str = "upstream";
const HEAD_PATH: &str = "head";
// TODO: do we have untracked?
const TRACKED_PATH: &str = "tracked";
const RELIC_IGNORE_PATH: &str = ".relic_ignore";

pub struct State {
    pub root_path: PathBuf,
    pub tracking_set: ContentSet,
    pub ignore_set: ContentSet,
}
impl State {
    pub fn initialise() -> Option<State> {
        // creates all relevant files & folders at specified root path
        None
    }

    pub fn construct(root_path: PathBuf) -> Option<State> {
        // load tracking and ignore set
        let relic_path = root_path.join(RELIC_PATH);
        if !relic_path.exists() {
            return None;
        }

        let branches_path = relic_path.join(BRANCHES_PATH);
        if !branches_path.exists() {
            return None;
        }

        let local_branches_path = branches_path.join(LOCAL_BRANCHES_PATH);
        let upstream_branches_path = branches_path.join(UPSTREAM_BRANCHES_PATH);
        if !(local_branches_path.exists() && upstream_branches_path.exists()) {
            return None;
        }

        let tracking_set = ContentSet::construct(&relic_path.join(TRACKED_PATH)).ok()?;
        let ignore_set = ContentSet::construct(&root_path.join(RELIC_IGNORE_PATH)).ok()?;

        Some(State {
            root_path,
            tracking_set,
            ignore_set,
        })
    }

    // // input: path to a file containing a singular oid
    // // output: Commit object from the oid
    // fn fetch_from_commit_file(&self, path: PathBuf) -> Result<Option<Commit>, RelicError> {
    //     if !path.exists() {
    //         return Err(RelicError::IOError(IOError::FileNoExist));
    //     }

    //     if let Ok(oid_raw) = fs::read(path) {
    //         if oid_raw.is_empty() {
    //             return Ok(None);
    //         }

    //         // let oid_raw: ObjectID = string_to_oid(str::from_utf8(&oid_raw).unwrap()).into();
    //         let oid_raw = match ObjectID::from_string(str::from_utf8(&oid_raw).unwrap()) {
    //             Some(o) => o,
    //             None => return Err(RelicError::ObjectID(super::error::ObjectID::InvalidID)),
    //         };

    //         match oid_raw.construct(&self.get_sanctum_path()) {
    //             Ok(c) => {
    //                 return match c {
    //                     Object::Commit(c) => Ok(Some(c)),
    //                     _ => Err(RelicError::ConfigurationIncorrect),
    //                 };
    //             }
    //             Err(_) => return Err(RelicError::ConfigurationIncorrect),
    //         }
    //     }
    //     Err(RelicError::IOError(IOError::FileCantOpen))
    // }

    pub fn fetch_local_head_commit(&self) -> Result<Option<Commit>, RelicError> {
        Branch::get_head(self, &BranchSource::Local)
            .and_then(|h| h.get_commit(&self.get_sanctum_path()))
    }

    pub fn fetch_upstream_head_commit(&self) -> Result<Option<Commit>, RelicError> {
        Branch::get_head(self, &BranchSource::Upstream)
            .and_then(|h| h.get_commit(&self.get_sanctum_path()))
    }

    pub fn update_tracking_set(&self) {
        let _ = fs::write(
            self.get_relic_path().join(TRACKED_PATH),
            self.tracking_set.serialise(),
        );
    }

    // #region paths
    pub fn get_relic_path(&self) -> PathBuf {
        self.root_path.join(RELIC_PATH)
    }

    pub fn get_branches_path(&self) -> PathBuf {
        self.get_relic_path().join(BRANCHES_PATH)
    }

    pub fn get_local_branches_path(&self) -> PathBuf {
        self.get_branches_path().join(LOCAL_BRANCHES_PATH)
    }

    pub fn get_upstream_branches_path(&self) -> PathBuf {
        self.get_branches_path().join(UPSTREAM_BRANCHES_PATH)
    }

    pub fn get_sanctum_path(&self) -> PathBuf {
        let s = self.get_relic_path().join(SANCTUM_PATH);
        if !s.exists() {
            // TODO: handle exceptions
            fs::create_dir(&s).unwrap();
        }
        s
    }

    pub fn get_head_path(&self) -> PathBuf {
        self.get_relic_path().join(HEAD_PATH)
    }
    // #endregion
}
