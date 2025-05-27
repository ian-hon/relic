use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

use crate::{
    content::{Content, Directory, File},
    content_set::ContentSet,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Change {
    pub container_modifications: Vec<ContainerModification>,
    pub modifications: Vec<Modification>,
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

        let mut result: Vec<String> = vec![];

        for c_m in &self.container_modifications {
            result.push(match c_m {
                ContainerModification::CreateDirectory(p, n) => {
                    format!(
                        "+ D {} {}",
                        urlencoding::encode(p).to_string(),
                        urlencoding::encode(n).to_string()
                    )
                }
                ContainerModification::DeleteDirectory(p, n) => {
                    format!(
                        "- D {} {}",
                        urlencoding::encode(p).to_string(),
                        urlencoding::encode(n).to_string()
                    )
                }
                ContainerModification::CreateFile(p, n) => {
                    format!(
                        "+ F {} {}",
                        urlencoding::encode(p).to_string(),
                        urlencoding::encode(n).to_string()
                    )
                }
                ContainerModification::DeleteFile(p, n) => {
                    format!(
                        "- F {} {}",
                        urlencoding::encode(p).to_string(),
                        urlencoding::encode(n).to_string()
                    )
                }
            });
        }

        result.push("=".to_string());

        let mut map = HashMap::new();
        for modification in &self.modifications {
            let path = match modification {
                Modification::Create(path, name, _, _) => (path.clone(), name.clone()),
                Modification::Delete(path, name, _, _) => (path.clone(), name.clone()),
            };
            map.entry(path).or_insert(vec![]).push(modification.clone());
        }

        let mut keys = map
            .iter()
            .map(|x| x.0.clone())
            .collect::<Vec<(String, String)>>();

        // map.sort_by_key(|x| x.0.clone());
        keys.sort();

        for (path, name) in keys {
            let modifications = map.get(&(path.clone(), name.clone())).unwrap();
            result.push(format!(
                "| {} {}",
                urlencoding::encode(&path).to_string(),
                urlencoding::encode(&name).to_string()
            ));
            for m in modifications {
                result.push(match m {
                    Modification::Create(_, _, line, content) => format!("+ {line} {content:?}"),
                    Modification::Delete(_, _, line, content) => format!("- {line} {content:?}"),
                })
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
        let mut container_section = true;

        let mut previous_file = None;
        for l in lines {
            if container_section && (l == "=") {
                container_section = false;
                continue;
            }
            let content = l.split(" ").collect::<Vec<&str>>();

            if container_section {
                let [species, container, parent, name] = *content.as_slice() else {
                    return None;
                };

                result
                    .container_modifications
                    .push(match (species, container) {
                        ("+", "D") => ContainerModification::CreateDirectory(
                            urlencoding::decode(parent).unwrap().to_string(),
                            urlencoding::decode(name).unwrap().to_string(),
                        ),
                        ("-", "D") => ContainerModification::DeleteDirectory(
                            urlencoding::decode(parent).unwrap().to_string(),
                            urlencoding::decode(name).unwrap().to_string(),
                        ),
                        ("+", "F") => ContainerModification::CreateFile(
                            urlencoding::decode(parent).unwrap().to_string(),
                            urlencoding::decode(name).unwrap().to_string(),
                        ),
                        ("-", "F") => ContainerModification::DeleteFile(
                            urlencoding::decode(parent).unwrap().to_string(),
                            urlencoding::decode(name).unwrap().to_string(),
                        ),
                        _ => {
                            println!("invalid c_mod");
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

                    previous_file = Some((parent.to_string(), name.to_string()));
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

                    if let Some((p, _)) = &previous_file {
                        if content.len() < 3 {
                            return None;
                        }
                    }

                    match &previous_file {
                        Some((p, n)) => match species {
                            "+" => {
                                let s = unescape::unescape(&content[2..].join(" ")).unwrap();

                                result.modifications.push(Modification::Create(
                                    urlencoding::decode(p).unwrap().to_string(),
                                    urlencoding::decode(n).unwrap().to_string(),
                                    line,
                                    s[1..s.len() - 1].to_string(),
                                ));
                            }
                            "-" => {
                                let s = unescape::unescape(&content[2..].join(" ")).unwrap();
                                result.modifications.push(Modification::Delete(
                                    urlencoding::decode(p).unwrap().to_string(),
                                    urlencoding::decode(n).unwrap().to_string(),
                                    line,
                                    s[1..s.len() - 1].to_string(),
                                ))
                            }
                            _ => {
                                return None;
                            }
                        },
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
            container_modifications: vec![],
            modifications: vec![],
        }
    }

    pub fn get_change(
        path: String,
        upstream_file: &File,
        current_file: &File,
    ) -> Vec<Modification> {
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
                ChangeTag::Delete => Modification::Delete(
                    path.clone(),
                    current_file.name.clone(),
                    change.old_index().unwrap(),
                    change.to_string().strip_suffix("\n").unwrap().to_string(),
                ),
                ChangeTag::Insert => Modification::Create(
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

    pub fn get_change_all(upstream: &Directory, current: &Directory, path: &Path) -> Change {
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
                Content::File(f) => {
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
                Content::File(f) => {
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
                container_modifications.push(ContainerModification::DeleteFile(
                    path.to_string_lossy().to_string(),
                    dir_name,
                ));
            } else {
                container_modifications.push(ContainerModification::DeleteDirectory(
                    path.to_string_lossy().to_string(),
                    dir_name.clone(),
                ));
                // traverse all children, add them to result as well
                let mut changes = Change::get_change_all(
                    match upstream_map.get(&(dir_name.clone(), false)).unwrap() {
                        Content::Directory(deleted_d) => deleted_d,
                        _ => panic!(),
                    },
                    &Directory::new(),
                    &path.join(dir_name.clone()),
                );
                container_modifications.append(&mut changes.container_modifications);
                modifications.append(&mut changes.modifications);
            }
        }
        //

        // for all created files, log them
        // for all created directories, log them and do the same for all children
        for (dir_name, is_file) in created {
            if is_file {
                // let p = path.join(dir_name.clone()).to_string_lossy().to_string();
                container_modifications.push(ContainerModification::CreateFile(
                    path.to_string_lossy().to_string(),
                    dir_name.clone(),
                ));
                // Modification::Create here
                modifications.append(&mut Change::get_change(
                    path.to_string_lossy().to_string(),
                    &File::new(),
                    match current_map.get(&(dir_name, true)).unwrap() {
                        Content::File(f) => f,
                        _ => panic!(),
                    },
                ))
            } else {
                // let p = path.join(dir_name.clone());
                container_modifications.push(ContainerModification::CreateDirectory(
                    path.to_string_lossy().to_string(),
                    dir_name.clone(),
                ));

                let mut changes = Change::get_change_all(
                    &Directory::new(),
                    match current_map.get(&(dir_name.clone(), false)).unwrap() {
                        Content::Directory(d) => d,
                        _ => panic!(),
                    },
                    &path.join(dir_name.clone()),
                );
                container_modifications.append(&mut changes.container_modifications);
                modifications.append(&mut changes.modifications);
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
                    container_modifications.append(&mut changes.container_modifications);
                    modifications.append(&mut changes.modifications);
                }
                Content::File(f) => {
                    let upstream_file = match upstream_map.get(&(f.name.clone(), true)) {
                        Some(c) => match c {
                            Content::File(f) => f,
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
            container_modifications,
            modifications,
        }
    }

    pub fn as_map(
        &self,
    ) -> (
        HashMap<String, HashSet<ContainerModification>>,
        HashMap<String, HashMap<String, Vec<Modification>>>,
    ) {
        // c_mod_map: map<parent_directory, Vec<changes>>
        // mod_map: map<parent_directory, map<file_name, Vec<changes>>>

        let mut c_mod_map = HashMap::new();
        for container_modification in &self.container_modifications {
            let path = match container_modification {
                ContainerModification::CreateDirectory(path, _)
                | ContainerModification::DeleteDirectory(path, _)
                | ContainerModification::CreateFile(path, _)
                | ContainerModification::DeleteFile(path, _) => path.clone(),
            };

            c_mod_map
                .entry(path)
                .or_insert(HashSet::new())
                .insert(container_modification.clone());
        }

        let mut mod_map = HashMap::new();
        for modification in &self.modifications {
            let (parent_directory, file_name) = match modification {
                Modification::Create(path, name, _, _) => (path.clone(), name.clone()),
                Modification::Delete(path, name, _, _) => (path.clone(), name.clone()),
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
            container_modifications: self
                .container_modifications
                .clone()
                .into_iter()
                .filter(|c_mod| match c_mod {
                    ContainerModification::CreateFile(p, n)
                    | ContainerModification::DeleteFile(p, n) => filter
                        .files
                        .contains(&PathBuf::from(p).join(n).to_string_lossy().to_string()),
                    ContainerModification::CreateDirectory(p, n)
                    | ContainerModification::DeleteDirectory(p, n) => filter
                        .directories
                        .contains(&PathBuf::from(p).join(n).to_string_lossy().to_string()),
                })
                .collect(),
            modifications: self
                .modifications
                .clone()
                .into_iter()
                .filter(|m| {
                    filter.files.contains(&match m {
                        Modification::Create(p, n, _, _) | Modification::Delete(p, n, _, _) => {
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
            container_modifications: self
                .container_modifications
                .iter()
                .map(|c| match c {
                    ContainerModification::CreateFile(p, n) => {
                        ContainerModification::DeleteFile(p.to_string(), n.to_string())
                    }
                    ContainerModification::CreateDirectory(p, n) => {
                        ContainerModification::DeleteDirectory(p.to_string(), n.to_string())
                    }
                    ContainerModification::DeleteFile(p, n) => {
                        ContainerModification::CreateFile(p.to_string(), n.to_string())
                    }
                    ContainerModification::DeleteDirectory(p, n) => {
                        ContainerModification::CreateDirectory(p.to_string(), n.to_string())
                    }
                })
                .collect::<Vec<ContainerModification>>(),
            modifications: self
                .modifications
                .iter()
                .map(|m| match m {
                    Modification::Create(p, f, l, t) => {
                        Modification::Delete(p.to_string(), f.to_string(), *l, t.to_string())
                    }
                    Modification::Delete(p, f, l, t) => {
                        Modification::Create(p.to_string(), f.to_string(), *l, t.to_string())
                    }
                })
                .collect::<Vec<Modification>>(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Modification {
    // creation/deletion of lines in files
    Create(
        String, // parent directory
        String, // file name
        usize,  // line
        String, // text
    ),
    Delete(
        String, // parent directory
        String, // file name
        usize,  // line
        String, // text
    ),
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContainerModification {
    // denote that parent doesnt exist?

    // creation/deletion of files & folders
    CreateDirectory(
        String, // parent directory
        String, // name
    ),
    DeleteDirectory(
        String, // parent directory
        String, // name
    ),

    CreateFile(
        String, // parent directory
        String, // name
    ),
    DeleteFile(
        String, // parent directory
        String, // name
    ),
}
