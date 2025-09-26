use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    core::{
        commit::Commit,
        content_set::{ContentSet, IgnoreSet, TrackingSet},
        modifications::Change,
        paths::{RELIC_PATH_IGNORE, RELIC_PATH_PENDING, RELIC_PATH_TRACKED, RELIC_PATH_UPSTREAM},
        Content, Tree, Blob, RelicInfo,
    },
    error::RelicError,
};

pub const DEFAULT_INFO: &str = r#"{
    "remote":"",
    "branch":"main"
}"#;
pub const DEFAULT_UPSTREAM: &str = r#"{
    "path": "",
    "name": "",
    "content": []
}"#;

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub info: RelicInfo,
    pub current: Tree,
    pub upstream: Tree,
    pub path: PathBuf,
    pub track_set: ContentSet,
    pub ignore_set: ContentSet,
}

impl State {
    pub fn empty() -> State {
        // needs to store current upstream commit
        // local commits assigned an id?
        State {
            info: RelicInfo::empty(),
            current: Tree::new(),
            upstream: Tree::new(),
            path: PathBuf::from(""),
            track_set: ContentSet::empty(),
            ignore_set: ContentSet::empty(),
        }
    }

    pub fn create(path: PathBuf) -> Result<State, RelicError> {
        let info = match RelicInfo::initialise() {
            Ok(r) => r,
            Err(e) => return Err(e),
        };

        let ignore_set =
            IgnoreSet::create(fs::read_to_string(RELIC_PATH_IGNORE).unwrap_or("".to_string()));

        let current =
            match State::content_at(&path.to_string_lossy().to_string(), &path, &ignore_set)? {
                Content::Directory(d) => d,
                _ => return Err(RelicError::ConfigurationIncorrect),
            };

        let upstream = match fs::read_to_string(RELIC_PATH_UPSTREAM) {
            Ok(data) => match Tree::deserialise(data) {
                Some(d) => d,
                // TODO : implement something better for this?
                None => Tree::new(), // None => return Err(RelicError::ConfigurationIncorrect),
            },
            Err(_) => return Err(RelicError::FileCantOpen),
        };

        let mut track_set: ContentSet = match fs::read_to_string(RELIC_PATH_TRACKED) {
            Ok(data) => TrackingSet::deserialise(data),
            Err(_) => return Err(RelicError::ConfigurationIncorrect),
        };

        track_set.directories = HashSet::from_iter(
            track_set
                .directories
                .difference(&ignore_set.directories)
                .map(|x| {
                    PathBuf::from(".")
                        .join(PathBuf::from(x))
                        .to_string_lossy()
                        .to_string()
                }),
        );
        track_set.files =
            HashSet::from_iter(track_set.files.difference(&ignore_set.files).map(|x| {
                PathBuf::from(".")
                    .join(PathBuf::from(x))
                    .to_string_lossy()
                    .to_string()
            }));

        Ok(State {
            info,
            current,
            upstream,
            path,
            track_set,
            ignore_set,
        })
    }

    pub fn content_at(
        file_name: &String,
        root_path: &PathBuf,
        ignore_set: &ContentSet,
    ) -> Result<Content, RelicError> {
        // get all files at path
        let paths = match fs::read_dir(root_path) {
            // let paths = match fs::read_dir(format!("./{}", root_path.clone())) {
            Ok(r) => r,
            Err(e) => {
                println!("state.rs (content_at) get all dirs : {root_path:?} : {e:?}");
                return Err(RelicError::FileCantOpen);
            }
        };

        let mut directory_contents = vec![];

        // iterate through them all
        for path in paths {
            match path {
                Ok(p) => {
                    let file_type = p.file_type().unwrap();
                    let file_name = p.file_name().into_string().unwrap();
                    let file_path = p.path();

                    // if file_name.starts_with(".") {
                    //     continue;
                    // }

                    if file_type.is_dir() {
                        if ignore_set.directories.contains(&file_name) {
                            continue;
                        }

                        match State::content_at(&file_name, &file_path, ignore_set) {
                            Ok(c) => {
                                directory_contents.push(c);
                            }
                            Err(e) => {
                                println!("state.rs (content_at) subtraverse : {e:?}");
                            }
                        }
                    } else if file_type.is_file() {
                        if ignore_set.files.contains(&file_name) {
                            continue;
                        }

                        match Blob::create(file_name, file_path) {
                            Ok(f) => {
                                directory_contents.push(Content::Blob(f));
                            }
                            _ => {}
                        }
                    } else if file_type.is_symlink() {
                        // TODO : decide what to do here
                        if ignore_set.files.contains(&file_name) {
                            continue;
                        }
                    }
                }
                Err(e) => {
                    println!("state.rs (content_at) read_dir : {e:?}");
                }
            }
        }

        // println!("CREATION : {root_path:?}");
        Ok(Content::Directory(Tree {
            path: root_path.clone(),
            name: file_name.clone(),
            content: directory_contents,
        }))
    }

    pub fn serialise_state(self: &State) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn deserialise_state(s: String) -> Option<State> {
        match serde_json::from_str(&s) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }

    // #region changes
    pub fn get_changes(&self) -> Change {
        Change::get_change_all(&self.upstream, &self.current, Path::new(&self.path))
    }
    // #endregion

    // #region upstream
    pub fn update_upstream(&mut self, tracked_content: &ContentSet) {
        // fully fill tracked_content
        // eg : "lorem/" -> ["lorem/ipsum", "lorem/dolor", "lorem/sit"]
        // traverse directories and fetch all children

        let tracked_content = tracked_content.clone().initialise(&mut self.current);

        // get changes
        // filter to only changes in the tracked_content content set
        let changes = self.get_changes().filter_changes(&tracked_content);

        // apply changes to current
        self.upstream.apply_changes(changes);
        let _ = fs::write(RELIC_PATH_UPSTREAM, self.upstream.serialise());
    }
    // #endregion

    // #region pending
    pub fn pending_add(&self, commit: Commit) {
        // TODO : use numbering for file name
        // who knows if two commits are created in the same nanosecond
        let _ = fs::write(
            format!("{RELIC_PATH_PENDING}/{}.diff", commit.timestamp),
            commit.serialise(),
        );
    }

    pub fn pending_get(&self) -> Vec<Commit> {
        let directories = if let Ok(d) = fs::read_dir(RELIC_PATH_PENDING) {
            d
        } else {
            return vec![];
        };

        let mut result = vec![];

        for d in directories {
            let d = if let Ok(d) = d { d } else { continue };
            let p = if let Ok(p) = fs::read_to_string(d.path()) {
                p
            } else {
                continue;
            };

            if let Some(c) = Commit::deserialise(p) {
                result.push(c);
            }
        }

        result.sort_by_key(|c| c.timestamp);

        result
    }
    // #endregion
}
