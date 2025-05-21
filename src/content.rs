use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use crate::{
    change::{Change, ContainerModification, Modification},
    error::RelicError,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Content {
    Directory(Directory),
    File(File),
}

#[derive(Debug)]
pub enum ContentMutRef<'a> {
    Directory(&'a mut Directory),
    File(&'a mut File),
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
            content: "".to_string(),
        }
    }

    pub fn create(name: String, path: PathBuf) -> Result<File, RelicError> {
        match fs::read_to_string(path) {
            Ok(content) => Ok(File {
                name: name,
                content: content,
            }),
            Err(_) => {
                // println!("Error creating file : {e}");
                Err(RelicError::FileCantOpen)
            }
        }
    }

    pub fn apply_changes(&mut self, modifications: &Vec<Modification>) {
        // TODO : investigate whether an additional newline is added to eof
        // BUG : when the file has only one line, diffs start to break
        //
        // content : ""
        // Create(,, 0, "something")
        // result : "something\nsomething"
        //
        // content : "something\nsomething"
        // Delete(,, 0)
        // result : ""

        // CHANGES ARE BEING APPLIED TO THE WRONG FILE
        // APPLY CHANGES TO UPSTREAM, NOT CURRENT

        // TODO : revise modification order

        // deletions first then creations?
        //      sorted largest to smallest
        // creations sorted smallest to largest?
        let mut lines = self
            .content
            .split("\n")
            .map(|x| x.to_string())
            .collect::<Vec<String>>();

        let mut modifications = modifications.clone();
        modifications.sort_by_key(|m| match m {
            Modification::Create(_, _, l, _) => *l as i128,
            Modification::Delete(_, _, l) => -(*l as i128),
        });

        for m in &modifications {
            match m {
                Modification::Create(_, _, line, content) => {
                    // insert at that line
                    lines.insert(*line, content.clone());
                }
                Modification::Delete(_, _, line) => {
                    // delete that line
                    lines.remove(*line);
                }
            }
        }

        self.content = lines.join("\n");
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Directory {
    pub path: PathBuf,
    pub name: String,
    pub content: Vec<Content>,
}

impl Directory {
    pub fn new() -> Directory {
        Directory {
            path: PathBuf::from(""),
            name: "".to_string(),
            content: vec![],
        }
    }

    pub fn deserialise(s: String) -> Option<Directory> {
        match serde_json::from_str(&s) {
            Ok(d) => Some(d),
            _ => None,
        }
    }

    pub fn serialise(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }

    pub fn apply_changes(&mut self, changes: Change) {
        let (c_mod_map, mod_map) = changes.as_map();
        let c_mod_map = Arc::new(Mutex::new(c_mod_map));

        // two pass
        // create/delete containers, then create/delete file content

        self.traverse(
            PathBuf::from("."),
            &|_, _, current| {
                if let ContentMutRef::Directory(d) = current {
                    // somehow denote that the parent does not yet exist,
                    // possibly recursively create directories where needed

                    // TODO : optimise the match arms
                    let mut c_mod_map_lock = c_mod_map.lock().unwrap();
                    if let Some(c_modifications) =
                        c_mod_map_lock.get(&d.path.to_string_lossy().to_string())
                    {
                        let c_clone = c_modifications.clone();

                        // deals with additions
                        d.content.append(&mut recursive_birth(
                            &PathBuf::from(d.path.clone()),
                            &mut c_mod_map_lock,
                        ));

                        let mut deleted_containers = HashSet::new();
                        // deals with subtractions
                        for c_mod in &c_clone {
                            match c_mod {
                                ContainerModification::DeleteDirectory(_, n) => {
                                    deleted_containers.insert(n);
                                }
                                ContainerModification::DeleteFile(_, n) => {
                                    deleted_containers.insert(n);
                                }
                                _ => {}
                            }
                        }

                        d.content = d
                            .content
                            .iter()
                            .filter(|x| {
                                !deleted_containers.contains(match x {
                                    Content::File(f) => &f.name,
                                    Content::Directory(d) => &d.name,
                                })
                            })
                            .map(|x| x.clone())
                            .collect::<Vec<Content>>();
                    }
                }
            },
            &Directory::new(),
        );

        self.traverse(
            PathBuf::from("."),
            &|path, _, current| {
                if let ContentMutRef::File(f) = current {
                    if let Some(modifications) = mod_map
                        .get(&path.to_string_lossy().to_string())
                        .map_or(None, |x| x.get(&f.name))
                    {
                        f.apply_changes(modifications);
                    }
                }
            },
            &self.clone(),
        );

        pub fn recursive_birth(
            parent_directory: &PathBuf,
            c_mod_map: &mut HashMap<String, HashSet<ContainerModification>>,
        ) -> Vec<Content> {
            // pass the new directory's parent directory
            let mut result = vec![];
            if let Some(c_modifications) =
                c_mod_map.get_mut(&parent_directory.to_string_lossy().to_string())
            {
                let c_clone = c_modifications.clone();
                for c in &c_clone {
                    c_modifications.remove(&c);
                }
                for c_mod in c_clone {
                    match c_mod {
                        ContainerModification::CreateDirectory(_, n) => {
                            result.push(Content::Directory(Directory {
                                path: parent_directory.join(n.clone()),
                                name: n.clone(),
                                content: recursive_birth(
                                    &parent_directory.join(n.clone()),
                                    c_mod_map,
                                ),
                            }));
                        }
                        ContainerModification::CreateFile(_, n) => {
                            result.push(Content::File(File {
                                name: n.clone(),
                                content: "".to_string(),
                            }))
                        }
                        _ => {}
                    }
                }
            }
            result
        }
    }

    pub fn traverse<F>(&mut self, root_path: PathBuf, func: &F, parent: &Directory)
    where
        // parent path, parent directory, current content
        F: Fn(&PathBuf, &Directory, ContentMutRef),
    {
        func(&root_path, &parent, ContentMutRef::Directory(self));

        let c = self.clone();
        for content in &mut self.content {
            match content {
                Content::Directory(d) => {
                    d.traverse(root_path.join(d.name.clone()), func, &c);
                }
                Content::File(f) => {
                    func(&root_path, &c, ContentMutRef::File(f));
                }
            }
        }
    }
}
