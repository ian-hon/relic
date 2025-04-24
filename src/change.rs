use std::{collections::{HashMap, HashSet}, path::Path};

use strum_macros::Display;

use crate::{content::{Content, Directory}, state::State};

#[derive(Debug)]
pub struct Change {
    pub container_modifications: Vec<ContainerModification>,
    pub modifications: Vec<Modification>
}
impl Change {
    pub fn serialise_changes() -> String {
        "".to_string()
    }
    
    pub fn deserialise_changes() -> String {
        "".to_string()
    }

    pub fn get_change(state: State) -> Change {


        Change {
            container_modifications: vec![],
            modifications: vec![],
        }
    }

    pub fn get_change_container(upstream: &Directory, current: &Directory, path: &Path) -> Vec<ContainerModification> {
        // assume that both current and previous have the same directory names
        // has to be bfs

        let mut current_set = HashSet::new();
        let mut current_map = HashMap::new();
        for c in &current.content {
            match c {
                Content::Directory(d) => {
                    current_set.insert((d.name.clone(), false));
                    current_map.insert((d.name.clone(), false), c);
                },
                Content::File(f) => {
                    current_set.insert((f.name.clone(), true));
                    current_map.insert((f.name.clone(), true), c);
                }
            }
        }

        let mut upstream_set = HashSet::new();
        let mut upstream_map = HashMap::new();
        for c in &upstream.content {
            match c {
                Content::Directory(d) => {
                    upstream_set.insert((d.name.clone(), false));
                    upstream_map.insert((d.name.clone(), false), c);
                },
                Content::File(f) => {
                    upstream_set.insert((f.name.clone(), true));
                    upstream_map.insert((f.name.clone(), true), c);
                }
            }
        }

        let deleted = upstream_set.difference(&current_set).map(|(n, t)| (n.to_string(), *t)).collect::<Vec<(String, bool)>>();
        let created = current_set.difference(&upstream_set).map(|(n, t)| (n.to_string(), *t)).collect::<Vec<(String, bool)>>();

        let mut result = vec![];
        for (d, is_file) in deleted {
            result.push(
                if is_file {
                    ContainerModification::DeleteFile(path.join(d.clone()).to_string_lossy().to_string())
                } else {
                    ContainerModification::DeleteDirectory(path.join(d.clone()).to_string_lossy().to_string())
                }
            );
        }
        for (d, is_file) in created {
            if is_file {
                result.push(
                    ContainerModification::CreateFile(path.join(d.clone()).to_string_lossy().to_string(), d)
                );
            } else {
                // let p = path.join(d.clone()).to_string_lossy().to_string();
                // result.push(
                //     ContainerModification::CreateDirectory(p.clone(), d.clone())
                // );
                // match current_map.get(&d).unwrap() {
                //     Content::Directory(dir) => {
                //         result.append(&mut Change::get_change_container(
                //             &Directory::new(),
                //             dir,
                //             Path::new(&p)
                //         ));
                //     },
                //     _ => {}
                // }
            }
        }

        for c in &current.content {
            match c {
                Content::Directory(d) => {
                    let p = path.join(d.name.clone()).to_string_lossy().to_string();
                    let upstream_directory = match upstream_map.get(&(d.name.clone(), false)) {
                        Some(u) => {
                            match u {
                                Content::Directory(u_d) => { u_d },
                                _ => panic!()
                            }
                        },
                        None => {
                            result.push(ContainerModification::CreateDirectory(p.clone(), d.name.clone()));
                            &Directory::new()
                        }
                    };

                    match current_map.get(&(d.name.clone(), false)).unwrap() {
                        Content::Directory(dir) => {
                            result.append(&mut Change::get_change_container(
                                upstream_directory,
                                dir,
                                Path::new(&p)
                            ));
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        result
    }
}

#[derive(Debug)]
pub enum Modification {
    // creation/deletion of lines in files
    Create(u128, Vec<u8>),
    Delete(u128)
}

#[derive(Debug)]
pub enum ContainerModification {
    // creation/deletion of files & folders
    // TODO : change so only path needed
    CreateDirectory(
        String, // path
        String // name
    ),
    DeleteDirectory(
        String // path
    ),

    CreateFile(
        String, // path
        String // name
    ),
    DeleteFile(
        String, // path
    )
}