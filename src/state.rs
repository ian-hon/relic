use std::fs;
use serde::{Deserialize, Serialize};

use crate::{content::{Content, Directory, File}, error::BonesError, ignore::IgnoreSet};

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub contents: Content
    // should always be Content::Directory
}

impl State {
    pub fn empty() -> State {
        State {
            contents: Content::Directory(Directory {
                name: "".to_string(),
                content: vec![]
            })
        }
    }

    pub fn create(path: String, ignore_set: &IgnoreSet) -> Result<State, BonesError> {
        let mut s = State::empty();
        match State::content_at(path.clone(), path, ignore_set) {
            Ok(c) => {
                s.contents = c;
                Ok(s)
            },
            Err(e) => {
                Err(e)
            }
        }
    }

    pub fn content_at(file_name: String, root_path: String, ignore_set: &IgnoreSet) -> Result<Content, BonesError> {
        // println!("started at {root_path}");

        // get all files at path
        let paths = match fs::read_dir(root_path.clone()) {
            Ok(r) => r,
            Err(e) => {
                println!("Error : {root_path} : {e:?}");
                return Err(BonesError::FileCantOpen);
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
                                println!("Error subtraverse : {e:?}");
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
                    println!("Traverse error : {e:?}");
                }
            }
        }

        Ok(Content::Directory(Directory {
            name: file_name,
            content: directory_contents
        }))
    }
}
