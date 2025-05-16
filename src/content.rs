use std::{collections::{HashMap, HashSet}, fs, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};

use crate::{change::{Change, ContainerModification, Modification}, error::RelicError, utils};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Content {
    Directory(Directory),
    File(File),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct File {
    pub name: String,
    pub content: String,
}

impl File {
    pub fn new() -> File {
        File {
            name: "".to_string(),
            content: "".to_string()
        }
    }

    pub fn create(name: String, path: PathBuf) -> Result<File, RelicError> {
        match fs::read_to_string(path) {
            Ok(content) => {
                Ok(File {
                    name: name,
                    content: content
                })
            },
            Err(_) => {
                // println!("Error creating file : {e}");
                Err(RelicError::FileCantOpen)
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Directory {
    pub path: PathBuf,
    pub name: String,
    pub content: Vec<Content>
}

impl Directory {
    pub fn new() -> Directory {
        Directory {
            path: PathBuf::from(""),
            name: "".to_string(),
            content: vec![]
        }
    }

    pub fn deserialise(s: String) -> Option<Directory> {
        match serde_json::from_str(&s) {
            Ok(d) => Some(d),
            _ => None
        }
    }

    pub fn serialise(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn apply_changes(&mut self, changes: Change) {
        let (c_mod_map, mod_map) = changes.as_map();

        // println!("{}", serde_json::to_string_pretty(&self).unwrap().to_string());

        println!("{}", utils::generate_tree(&self));

        self.traverse(PathBuf::from("."), &c_mod_map, &mod_map, |path, parent, current, c_mod_map, mod_map| {
            match current {
                Content::Directory(d) => {
                    // somehow denote that the parent does not yet exist,
                    // possibly recursively create directories where needed

                    // TODO : optimise the match arms

                    if let Some(c_modifications) = c_mod_map.get(&d.path.to_string_lossy().to_string()) {
                        // deals with additions
                        d.content.append(&mut recursive_birth(&PathBuf::from(d.path.clone()), &c_mod_map));

                        let mut deleted_containers = HashSet::new();
                        // deals with subtractions
                        for c_mod in c_modifications {
                            match c_mod {
                                ContainerModification::DeleteDirectory(_, n) => {
                                    deleted_containers.insert(n);
                                }
                                ContainerModification::DeleteFile(_, n) => {
                                    deleted_containers.insert(n);
                                },
                                _ => {}
                            }
                        }

                        d.content = d.content
                            .iter()
                            .filter(|x|
                                !deleted_containers
                                    .contains(match x {
                                        Content::File(f) => &f.name,
                                        Content::Directory(d) => &d.name
                                    })
                            )
                            .map(|x| x.clone())
                            .collect::<Vec<Content>>();
                    }


                    // match c_mod_map.get(&path.to_string_lossy().to_string()) {
                    //     Some(c_m) => {
                    //         println!("C_M : {c_m:?}");
                    //         for c_mod in c_m {
                    //             let (parent_directory, container_name) = match c_mod {
                    //                 ContainerModification::CreateFile(p, n) => (p, n),
                    //                 ContainerModification::CreateDirectory(p, n) => (p, n),
                    //                 ContainerModification::DeleteFile(p, n) => (p, n),
                    //                 ContainerModification::DeleteDirectory(p, n) => (p, n)
                    //             };
                    //             if c_mod_map.contains_key(parent_directory) {
                    //                 d.content.append(&mut recursive_birth(&PathBuf::from(parent_directory), &c_mod_map));
                    //             }
                    //         }
                    //     },
                    //     None => {}
                    // }
                },
                Content::File(f) => {

                }
            }

            // println!("{} -> {} ({path:?})", parent.name, match current { Content::Directory(d) => d.name.clone(), Content::File(f) => f.name.clone() });
        });

        println!("{}", utils::generate_tree(&self));

        pub fn recursive_birth(parent_directory: &PathBuf, c_mod_map: &HashMap<String, Vec<ContainerModification>>) -> Vec<Content> {
            // pass the new directory's parent directory
            let mut result = vec![];
            match c_mod_map.get(&parent_directory.to_string_lossy().to_string()) {
                Some(c_modifications) => {
                    for c_mod in c_modifications {
                        match c_mod {
                            ContainerModification::CreateDirectory(p, n) => {
                                result.push(Content::Directory(Directory {
                                    path: parent_directory.join(n.clone()),
                                    name: n.clone(),
                                    content: recursive_birth(
                                        &parent_directory.join(n.clone()),
                                        c_mod_map
                                    )
                                }));
                            },
                            ContainerModification::CreateFile(p, n) => {
                                result.push(Content::File(File {
                                    name: n.clone(),
                                    content: "".to_string()
                                }))
                            },
                            _ => {}
                        }
                    }
                },
                None => {}
            }
            result
        }
    }

    pub fn traverse(&mut self, root_path: PathBuf, c_mod_map: &HashMap<String, Vec<ContainerModification>>, mod_map: &HashMap<String, HashMap<String, Vec<Modification>>>, func: fn(&PathBuf, &Directory, &mut Content, &HashMap<String, Vec<ContainerModification>>, &HashMap<String, HashMap<String, Vec<Modification>>>)) {
        // greedily consume nodes -> not possible, cant hold multiple mutables references

        let c = &self.clone();
        for content in &mut self.content {
            match content {
                Content::Directory(d) => {
                    // TODO : revise whether order of operations is correct
                    d.traverse(root_path.join(d.name.clone()), c_mod_map, mod_map, func);
                    func(&root_path.join(d.name.clone()), &c, content, &c_mod_map, &mod_map);
                },
                Content::File(f) => {

                }
            }
        }
    }
}
