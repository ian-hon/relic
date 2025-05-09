use std::fs;
use serde::{Deserialize, Serialize};

use crate::{content::{Content, Directory, File}, error::RelicError, ignore::IgnoreSet};

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub current: Directory,
    pub upstream: Directory
}

impl State {
    pub fn empty() -> State {
        State {
            current: Directory {
                name: "".to_string(),
                content: vec![]
            },
            upstream: Directory {
                name: "".to_string(),
                content: vec![]
            }
        }
    }

    pub fn create(path: String, ignore_set: &IgnoreSet) -> Result<State, RelicError> {
        let mut s = State::empty();
        match State::content_at(path.clone(), path, ignore_set)? {
            Content::Directory(d) => {
                s.current = d;
                Ok(s)
            },
            _ => {
                Err(RelicError::ConfigurationIncorrect)
            }
        }

        // fetch upstream

    }

    pub fn content_at(file_name: String, root_path: String, ignore_set: &IgnoreSet) -> Result<Content, RelicError> {
        // println!("started at {root_path}");

        // get all files at path
        let paths = match fs::read_dir(format!("./{}", root_path.clone())) {
            Ok(r) => r,
            Err(e) => {
                println!("state.rs (content_at) get all dirs : {root_path} : {e:?}");
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
                    let file_path = p.path().to_string_lossy().to_string();

                    if file_name.starts_with(".") {
                        continue;
                    }

                    if file_type.is_dir() {
                        if ignore_set.dir_ignore.contains(&file_name) {
                            continue;
                        }

                        match State::content_at(file_name, file_path, ignore_set) {
                            Ok(c) => {
                                directory_contents.push(c);
                            },
                            Err(e) => {
                                println!("state.rs (content_at) subtraverse : {e:?}");
                            }
                        }
                    } else if file_type.is_file() {
                        if ignore_set.file_ignore.contains(&file_name) {
                            continue;
                        }

                        match File::create(file_name, file_path) {
                            Ok(f) => {
                                directory_contents.push(Content::File(f));
                            },
                            _ => {}
                        }
                    } else if file_type.is_symlink() {
                        // TODO : decide what to do here
                        if ignore_set.file_ignore.contains(&file_name) {
                            continue;
                        }
                    }
                },
                Err(e) => {
                    println!("state.rs (content_at) read_dir : {e:?}");
                }
            }
        }

        Ok(Content::Directory(Directory {
            name: file_name,
            content: directory_contents
        }))
    }

    pub fn serialise_state(self: &State) -> String {
        serde_json::to_string(self).unwrap()
    }
    
    pub fn deserialise_state(s: String) -> Option<State> {
        match serde_json::from_str(&s) {
            Ok(s) => {
                Some(s)
            },
            Err(_) => {
                None
            }
        }
    }
}
