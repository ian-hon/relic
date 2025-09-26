use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use crate::core::{
    modifications::{self, Change},
    Blob, Content, ContentMutRef,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tree {
    pub path: PathBuf,
    pub name: String,
    pub content: Vec<Content>,
}

impl Tree {
    pub fn new() -> Tree {
        Tree {
            path: PathBuf::from(""),
            name: "".to_string(),
            content: vec![],
        }
    }

    pub fn get_hash(&self) -> String {
        sha256::digest(serde_json::to_string(&self).unwrap())
    }

    pub fn deserialise(s: String) -> Option<Tree> {
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
                if let ContentMutRef::Tree(t) = current {
                    // somehow denote that the parent does not yet exist,
                    // possibly recursively create trees where needed

                    // TODO : optimise the match arms
                    let mut c_mod_map_lock = c_mod_map.lock().unwrap();
                    if let Some(c_modifications) =
                        c_mod_map_lock.get(&t.path.to_string_lossy().to_string())
                    {
                        let c_clone = c_modifications.clone();

                        // deals with additions
                        t.content.append(&mut recursive_birth(
                            &PathBuf::from(t.path.clone()),
                            &mut c_mod_map_lock,
                        ));

                        let mut deleted_containers = HashSet::new();
                        // deals with subtractions
                        for c_mod in &c_clone {
                            match c_mod {
                                modifications::Tree::DeleteTree(_, n) => {
                                    deleted_containers.insert(n);
                                }
                                modifications::Tree::DeleteBlob(_, n) => {
                                    deleted_containers.insert(n);
                                }
                                _ => {}
                            }
                        }

                        t.content = t
                            .content
                            .iter()
                            .filter(|x| {
                                !deleted_containers.contains(match x {
                                    Content::Blob(b) => &b.name,
                                    Content::Tree(t) => &t.name,
                                })
                            })
                            .map(|x| x.clone())
                            .collect::<Vec<Content>>();
                    }
                }
            },
            &Tree::new(),
        );

        self.traverse(
            PathBuf::from("."),
            &|path, _, current| {
                if let ContentMutRef::Blob(f) = current {
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
            c_mod_map: &mut HashMap<String, HashSet<modifications::Tree>>,
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
                        modifications::Tree::CreateTree(_, n) => {
                            result.push(Content::Tree(Tree {
                                path: parent_directory.join(n.clone()),
                                name: n.clone(),
                                content: recursive_birth(
                                    &parent_directory.join(n.clone()),
                                    c_mod_map,
                                ),
                            }));
                        }
                        modifications::Tree::CreateBlob(_, n) => result.push(Content::Blob(Blob {
                            name: n.clone(),
                            content: "".to_string(),
                        })),
                        _ => {}
                    }
                }
            }
            result
        }
    }

    pub fn unapply_changes(&mut self, changes: Change) {
        // TODO : test if 100% reliable
        let changes = changes.inverse();
        self.apply_changes(changes);
    }

    pub fn traverse<F>(&mut self, root_path: PathBuf, func: &F, parent: &Tree)
    where
        // parent path, parent tree, current content
        F: Fn(&PathBuf, &Tree, ContentMutRef),
    {
        func(&root_path, &parent, ContentMutRef::Tree(self));

        let c = self.clone();
        for content in &mut self.content {
            match content {
                Content::Tree(t) => {
                    t.traverse(root_path.join(t.name.clone()), func, &c);
                }
                Content::Blob(b) => {
                    func(&root_path, &c, ContentMutRef::Blob(b));
                }
            }
        }
    }
}
