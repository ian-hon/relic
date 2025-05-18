use std::{collections::{HashMap, HashSet}, fs, path::{Path, PathBuf}, sync::Arc};

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

    pub fn apply_changes(&mut self, modifications: &Vec<Modification>) {
        // TODO : investigate whether an additional newline is added to eof
        let mut lines = self.content.split("\n").map(|x| x.to_string()).collect::<Vec<String>>();

        // println!("({}) BEFORE : {}\n{}", self.name, sha256::digest(&self.content), self.content);

        for m in modifications {
            match m {
                Modification::Create(_, _, line, content) => {
                    // insert at that line
                    lines.insert(*line, content.clone());
                },
                Modification::Delete(_, _, line) => {
                    // delete that line
                    lines.remove(*line);
                }
            }
        }

        self.content = lines.join("\n");

        // println!("({}) AFTER : {}\n{}", self.name, sha256::digest(&self.content), self.content);
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

        // println!("{}", utils::generate_tree(&self));

        // two pass
        // create/delete containers, then create/delete file content

        self.traverse(
            PathBuf::from("."),
            &|_, _, current| {
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
                },
                _ => {}
            }

            // println!("{} -> {} ({path:?})", parent.name, match current { Content::Directory(d) => d.name.clone(), Content::File(f) => f.name.clone() });
        });

        self.traverse(
            PathBuf::from("."),
            &|path, _, current| {
            match current {
                Content::File(f) => {
                    // THIS IS WHAT TO DO NEXT
                    if let Some(modifications) = mod_map
                        .get(&path.to_string_lossy().to_string())
                        .map_or(None, |x| x.get(&f.name)) {
                        // println!("{modifications:?}");
                        f.apply_changes(modifications);
                    }
                },
                _ => {}
            }

            // println!("{} -> {} ({path:?})", parent.name, match current { Content::Directory(d) => d.name.clone(), Content::File(f) => f.name.clone() });
        });

        // println!("{}", utils::generate_tree(&self));

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

    // pub fn traverse(&mut self, root_path: PathBuf, c_mod_map: &HashMap<String, Vec<ContainerModification>>, mod_map: &HashMap<String, HashMap<String, Vec<Modification>>>, func: fn(&PathBuf, &Directory, &mut Content, &HashMap<String, Vec<ContainerModification>>, &HashMap<String, HashMap<String, Vec<Modification>>>)) {
    pub fn traverse<F>(&mut self, root_path: PathBuf, func: &F)
    where
        F: Fn(&PathBuf, &Directory, &mut Content)
    {
        let c = self.clone();
        for content in &mut self.content {
            match content {
                Content::Directory(d) => {
                    func(&root_path.join(d.name.clone()), &c, content);
                },
                Content::File(_) => {
                    func(&root_path, &c, content);
                }
            }
            if let Content::Directory(d) = content {
                d.traverse(root_path.join(d.name.clone()), func);
            }
        }
    }
}
