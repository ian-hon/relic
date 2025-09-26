use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

use crate::core::{content_set::ContentSet, modifications, Blob, Content, Tree};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Change {
    pub trees: Vec<modifications::Tree>,
    pub blobs: Vec<modifications::Blob>,
}
impl Change {
    pub fn get_hash(&self) -> String {
        sha256::digest(self.serialise_changes())
    }

    pub fn serialise_changes(&self) -> String {
        // + D . src
        // + F .%2Fsrc utils.rs
        // + F .%2Fsrc branch.rs
        // =
        // | .%2Fsrc content.rs
        // + 0 "use std::{collections::{HashMap, HashSet}, fs, path::{Path, PathBuf}, sync::{Arc, Mutex}};"
        // + 1 ""

        // final result string
        let mut result: Vec<String> = vec![];

        for tree in &self.trees {
            result.push(tree.serialise());
        }

        result.push("=".to_string()); // container and file section separator

        let mut blob_sections = HashMap::new();
        for blob in &self.blobs {
            blob_sections
                .entry(blob.extract_path())
                .or_insert(vec![])
                .push(blob.clone());
        }

        let mut keys = blob_sections
            .iter()
            .map(|x| x.0.clone())
            .collect::<Vec<(String, String)>>();

        keys.sort();

        for (path, name) in keys {
            let modifications = blob_sections.get(&(path.clone(), name.clone())).unwrap();
            result.push(format!(
                "| {} {}",
                urlencoding::encode(&path).to_string(),
                urlencoding::encode(&name).to_string()
            ));
            for blob in modifications {
                result.push(blob.extract_change());
            }
        }

        result.join("\n")
    }

    pub fn deserialise_changes(s: String) -> Option<Change> {
        // + D . src
        // + F .%2Fsrc utils.rs
        // + F .%2Fsrc branch.rs
        // =
        // | .%2Fsrc content.rs
        // + 0 "use std::{collections::{HashMap, HashSet}, fs, path::{Path, PathBuf}, sync::{Arc, Mutex}};"
        // + 1 ""

        let lines = s
            .split("\n")
            .map(|x| x.to_string())
            .collect::<Vec<String>>();

        let mut result = Change::empty();
        let mut tree_section = true;

        let mut previous_blob = None;
        for l in lines {
            if tree_section && (l == "=") {
                tree_section = false;
                continue;
            }
            let content = l.split(" ").collect::<Vec<&str>>();

            if tree_section {
                let [species, container, parent, name] = *content.as_slice() else {
                    return None;
                };

                result.trees.push(match (species, container) {
                    ("+", "D") => modifications::Tree::CreateTree(
                        urlencoding::decode(parent).unwrap().to_string(),
                        urlencoding::decode(name).unwrap().to_string(),
                    ),
                    ("-", "D") => modifications::Tree::DeleteTree(
                        urlencoding::decode(parent).unwrap().to_string(),
                        urlencoding::decode(name).unwrap().to_string(),
                    ),
                    ("+", "F") => modifications::Tree::CreateBlob(
                        urlencoding::decode(parent).unwrap().to_string(),
                        urlencoding::decode(name).unwrap().to_string(),
                    ),
                    ("-", "F") => modifications::Tree::DeleteBlob(
                        urlencoding::decode(parent).unwrap().to_string(),
                        urlencoding::decode(name).unwrap().to_string(),
                    ),
                    _ => {
                        println!("invalid tree");
                        return None;
                    }
                });
            } else {
                if content[0] == "|" {
                    // | .%2Fsrc content.rs
                    let [_, parent, name] = *content.as_slice() else {
                        println!("invalid file header");
                        return None;
                    };

                    previous_blob = Some((parent.to_string(), name.to_string()));
                } else {
                    // + 0 "use std::{collections::{HashMap, HashSet}, fs, path::{Path, PathBuf}, sync::{Arc, Mutex}};"
                    if content.len() < 2 {
                        println!("invalid change line");
                        return None;
                    }

                    let species = content[0];
                    let line = match content[1].parse::<usize>() {
                        Ok(i) => i,
                        _ => {
                            println!("invalid line index");
                            return None;
                        }
                    };

                    match &previous_blob {
                        Some((p, n)) => {
                            let decoded_path = urlencoding::decode(p).unwrap().to_string();
                            let decoded_name = urlencoding::decode(n).unwrap().to_string();
                            let s = unescape::unescape(&content[2..].join(" ")).unwrap();
                            let content_text = s[1..s.len() - 1].to_string();

                            match species {
                                "+" => {
                                    result.blobs.push(modifications::Blob::Create(
                                        decoded_path,
                                        decoded_name,
                                        line,
                                        content_text,
                                    ));
                                }
                                "-" => {
                                    result.blobs.push(modifications::Blob::Delete(
                                        decoded_path,
                                        decoded_name,
                                        line,
                                        content_text,
                                    ));
                                }
                                _ => {
                                    return None;
                                }
                            }
                        }
                        None => {
                            return None;
                        }
                    }
                }
            }
        }

        Some(result)
    }

    pub fn empty() -> Change {
        Change {
            trees: vec![],
            blobs: vec![],
        }
    }

    pub fn get_change(
        path: String,
        upstream_file: &Blob,
        current_file: &Blob,
    ) -> Vec<modifications::Blob> {
        // https://blog.jcoglan.com/2017/02/15/the-myers-diff-algorithm-part-2/
        // for our change algorithm, we will be using myers diff algorithm
        // basically a shortest distance problem, with downwards, rightwards and diagonal directions as movement choices
        // (note that diagonal movements do not contribute towards the distance)

        // similar does not handle newlines at eof well at all
        // this is the workaround for it
        let upstream = format!("{}\n", upstream_file.content.clone());
        let current = format!("{}\n", current_file.content.clone());

        // TODO : compare hashes instead of files
        if upstream == current {
            return vec![];
        }

        let mut result = vec![];
        let diff = TextDiff::from_lines(&upstream, &current);

        for change in diff.iter_all_changes().filter_map(|c| match c.tag() {
            ChangeTag::Equal => None,
            _ => Some(c),
        }) {
            result.push(match change.tag() {
                ChangeTag::Delete => modifications::Blob::Delete(
                    path.clone(),
                    current_file.name.clone(),
                    change.old_index().unwrap(),
                    change.to_string().strip_suffix("\n").unwrap().to_string(),
                ),
                ChangeTag::Insert => modifications::Blob::Create(
                    path.clone(),
                    current_file.name.clone(),
                    change.new_index().unwrap(),
                    change.to_string().strip_suffix("\n").unwrap().to_string(),
                ),
                _ => panic!(),
            })
        }

        result
    }

    pub fn get_change_all(upstream: &Tree, current: &Tree, path: &Path) -> Change {
        // assume that both current and previous have the same directory names
        // has to be bfs

        // initialise current state set
        let mut current_set = HashSet::new();
        let mut current_map = HashMap::new();
        for c in &current.content {
            match c {
                Content::Directory(d) => {
                    current_set.insert((d.name.clone(), false));
                    current_map.insert((d.name.clone(), false), c);
                }
                Content::Blob(f) => {
                    current_set.insert((f.name.clone(), true));
                    current_map.insert((f.name.clone(), true), c);
                }
            }
        }
        //

        // initialise upstream state set
        let mut upstream_set = HashSet::new();
        let mut upstream_map = HashMap::new();
        for c in &upstream.content {
            match c {
                Content::Directory(d) => {
                    upstream_set.insert((d.name.clone(), false));
                    upstream_map.insert((d.name.clone(), false), c);
                }
                Content::Blob(f) => {
                    upstream_set.insert((f.name.clone(), true));
                    upstream_map.insert((f.name.clone(), true), c);
                }
            }
        }
        //

        // use set differences to determine file and directory creation or deletion
        let deleted = upstream_set
            .difference(&current_set)
            .map(|(n, t)| (n.to_string(), *t))
            .collect::<Vec<(String, bool)>>();
        let created = current_set
            .difference(&upstream_set)
            .map(|(n, t)| (n.to_string(), *t))
            .collect::<Vec<(String, bool)>>();
        //

        // for all deleted files, log them
        // for all deleted directories, log them and do the same for all children
        let mut container_modifications = vec![];
        let mut modifications = vec![];
        for (dir_name, is_file) in deleted {
            if is_file {
                container_modifications.push(modifications::Tree::DeleteBlob(
                    path.to_string_lossy().to_string(),
                    dir_name,
                ));
            } else {
                container_modifications.push(modifications::Tree::DeleteTree(
                    path.to_string_lossy().to_string(),
                    dir_name.clone(),
                ));
                // traverse all children, add them to result as well
                let mut changes = Change::get_change_all(
                    match upstream_map.get(&(dir_name.clone(), false)).unwrap() {
                        Content::Directory(deleted_d) => deleted_d,
                        _ => panic!(),
                    },
                    &Tree::new(),
                    &path.join(dir_name.clone()),
                );
                container_modifications.append(&mut changes.trees);
                modifications.append(&mut changes.blobs);
            }
        }
        //

        // for all created files, log them
        // for all created directories, log them and do the same for all children
        for (dir_name, is_file) in created {
            if is_file {
                // let p = path.join(dir_name.clone()).to_string_lossy().to_string();
                container_modifications.push(modifications::Tree::CreateBlob(
                    path.to_string_lossy().to_string(),
                    dir_name.clone(),
                ));
                // modifications::File::Create here
                modifications.append(&mut Change::get_change(
                    path.to_string_lossy().to_string(),
                    &Blob::new(),
                    match current_map.get(&(dir_name, true)).unwrap() {
                        Content::Blob(f) => f,
                        _ => panic!(),
                    },
                ))
            } else {
                // let p = path.join(dir_name.clone());
                container_modifications.push(modifications::Tree::CreateTree(
                    path.to_string_lossy().to_string(),
                    dir_name.clone(),
                ));

                let mut changes = Change::get_change_all(
                    &Tree::new(),
                    match current_map.get(&(dir_name.clone(), false)).unwrap() {
                        Content::Directory(d) => d,
                        _ => panic!(),
                    },
                    &path.join(dir_name.clone()),
                );
                container_modifications.append(&mut changes.trees);
                modifications.append(&mut changes.blobs);
            }
        }

        for content in &current.content {
            match content {
                Content::Directory(directory) => {
                    // get the matching upstream directory
                    // if it doesnt exist, that means the content is new and can be ignored
                    // we ignore it because we have already logged it in the section above
                    let p = path.join(directory.name.clone());
                    let upstream_directory =
                        match upstream_map.get(&(directory.name.clone(), false)) {
                            Some(u) => match u {
                                Content::Directory(u_d) => u_d,
                                _ => panic!(),
                            },
                            _ => {
                                continue;
                            }
                        };
                    //

                    let mut changes = Change::get_change_all(upstream_directory, directory, &p);
                    container_modifications.append(&mut changes.trees);
                    modifications.append(&mut changes.blobs);
                }
                Content::Blob(f) => {
                    let upstream_file = match upstream_map.get(&(f.name.clone(), true)) {
                        Some(c) => match c {
                            Content::Blob(f) => f,
                            _ => panic!(),
                        },
                        None => {
                            continue;
                        }
                    };

                    modifications.append(&mut Change::get_change(
                        path.to_string_lossy().to_string(),
                        &upstream_file,
                        &f,
                    ));
                }
            }
        }

        Change {
            trees: container_modifications,
            blobs: modifications,
        }
    }

    pub fn as_map(
        &self,
    ) -> (
        HashMap<String, HashSet<modifications::Tree>>,
        HashMap<String, HashMap<String, Vec<modifications::Blob>>>,
    ) {
        // c_mod_map: map<parent_directory, Vec<changes>>
        // mod_map: map<parent_directory, map<file_name, Vec<changes>>>

        let mut c_mod_map = HashMap::new();
        for container_modification in &self.trees {
            let path = match container_modification {
                modifications::Tree::CreateTree(path, _)
                | modifications::Tree::DeleteTree(path, _)
                | modifications::Tree::CreateBlob(path, _)
                | modifications::Tree::DeleteBlob(path, _) => path.clone(),
            };

            c_mod_map
                .entry(path)
                .or_insert(HashSet::new())
                .insert(container_modification.clone());
        }

        let mut mod_map = HashMap::new();
        for modification in &self.blobs {
            let (parent_directory, file_name) = match modification {
                modifications::Blob::Create(path, name, _, _) => (path.clone(), name.clone()),
                modifications::Blob::Delete(path, name, _, _) => (path.clone(), name.clone()),
            };
            mod_map
                .entry(parent_directory)
                .or_insert(HashMap::new())
                .entry(file_name)
                .or_insert(vec![])
                .push(modification.clone());
        }

        (c_mod_map, mod_map)
    }

    pub fn filter_changes(&self, filter: &ContentSet) -> Change {
        Change {
            trees: self
                .trees
                .clone()
                .into_iter()
                .filter(|c_mod| match c_mod {
                    modifications::Tree::CreateBlob(p, n)
                    | modifications::Tree::DeleteBlob(p, n) => filter
                        .files
                        .contains(&PathBuf::from(p).join(n).to_string_lossy().to_string()),
                    modifications::Tree::CreateTree(p, n)
                    | modifications::Tree::DeleteTree(p, n) => filter
                        .directories
                        .contains(&PathBuf::from(p).join(n).to_string_lossy().to_string()),
                })
                .collect(),
            blobs: self
                .blobs
                .clone()
                .into_iter()
                .filter(|m| {
                    filter.files.contains(&match m {
                        modifications::Blob::Create(p, n, _, _)
                        | modifications::Blob::Delete(p, n, _, _) => {
                            PathBuf::from(p).join(n).to_string_lossy().to_string()
                        }
                    })
                })
                .collect(),
        }
    }

    pub fn inverse(&self) -> Change {
        // returns inverse of the change
        // all additions are deletions and vice versa

        // the order does not follow the optimised/intuitive format
        // additions will appear before deletions if inversed
        // but relic will always apply changes in the correct order regardless

        Change {
            trees: self
                .trees
                .iter()
                .map(|c| match c {
                    modifications::Tree::CreateBlob(p, n) => {
                        modifications::Tree::DeleteBlob(p.to_string(), n.to_string())
                    }
                    modifications::Tree::CreateTree(p, n) => {
                        modifications::Tree::DeleteTree(p.to_string(), n.to_string())
                    }
                    modifications::Tree::DeleteBlob(p, n) => {
                        modifications::Tree::CreateBlob(p.to_string(), n.to_string())
                    }
                    modifications::Tree::DeleteTree(p, n) => {
                        modifications::Tree::CreateTree(p.to_string(), n.to_string())
                    }
                })
                .collect::<Vec<modifications::Tree>>(),
            blobs: self
                .blobs
                .iter()
                .map(|m| match m {
                    modifications::Blob::Create(p, f, l, t) => {
                        modifications::Blob::Delete(p.to_string(), f.to_string(), *l, t.to_string())
                    }
                    modifications::Blob::Delete(p, f, l, t) => {
                        modifications::Blob::Create(p.to_string(), f.to_string(), *l, t.to_string())
                    }
                })
                .collect::<Vec<modifications::Blob>>(),
        }
    }
}
