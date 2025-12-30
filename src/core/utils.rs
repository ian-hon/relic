use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, Utc};

use crate::core::{modifications, Blob, Content, Tree};

impl Blob {
    pub fn get_blame_header(
        &self,
        modifications: &HashMap<String, bool>,
        blob_info: &Vec<modifications::Blob>,
    ) -> String {
        // returns:
        // (-) earth
        // (+) mars
        // venus [+10, -10]

        let mod_type: Option<bool> = modifications.get(&self.name).copied();

        format!(
            "{}{} {}",
            match mod_type {
                Some(m) => {
                    if m {
                        "(+) "
                    } else {
                        "(-) "
                    }
                }
                None => "",
            },
            self.name.clone(),
            if blob_info.is_empty() {
                "".to_string()
            } else {
                format!(
                    "[+{}, -{}]",
                    blob_info
                        .iter()
                        .filter(|b| match b {
                            modifications::Blob::Create(_, _, _, _) => true,
                            _ => false,
                        })
                        .count(),
                    blob_info
                        .iter()
                        .filter(|b| match b {
                            modifications::Blob::Delete(_, _, _, _) => true,
                            _ => false,
                        })
                        .count(),
                )
            }
        )
    }
}

pub fn generate_blame_tree(
    tree: &Tree,
    tree_map: &HashMap<String, HashSet<modifications::Tree>>,
    blob_map: &HashMap<String, HashMap<String, Vec<modifications::Blob>>>,
) -> String {
    return generate_blame_subtree(
        &Content::Tree(tree.clone()),
        PathBuf::from("."),
        tree_map,
        blob_map,
    );
}

pub fn generate_blame_subtree(
    c: &Content,
    path: PathBuf,
    tree_map: &HashMap<String, HashSet<modifications::Tree>>,
    blob_map: &HashMap<String, HashMap<String, Vec<modifications::Blob>>>,
) -> String {
    let mut result = vec![];

    let modifications =
        tree_map
            .get(&path.to_string_lossy().to_string())
            .map_or(HashMap::new(), |h| {
                h.into_iter()
                    .map(|v| match v {
                        modifications::Tree::CreateTree(_, n)
                        | modifications::Tree::CreateBlob(_, n) => (n.to_string(), true),
                        modifications::Tree::DeleteTree(_, n)
                        | modifications::Tree::DeleteBlob(_, n) => (n.to_string(), false),
                    })
                    .collect::<HashMap<String, bool>>()
            });

    match c {
        Content::Tree(t) => {
            let name = t.name.clone();
            let mut r = vec![name];
            if t.content.len() >= 1 {
                let length = t.content.len() - 1;
                for (index, i) in t.content.iter().enumerate() {
                    let mut p = path.clone();
                    if !t.name.is_empty() {
                        p = path.join(t.name.clone());
                    }
                    for (inner_index, line) in generate_blame_subtree(i, p, tree_map, blob_map)
                        .split("\n")
                        .enumerate()
                    {
                        r.push(format!(
                            " {} {line}",
                            if index == length {
                                if inner_index == 0 {
                                    "└"
                                } else {
                                    ""
                                }
                            } else {
                                if inner_index == 0 {
                                    "├"
                                } else {
                                    "│"
                                }
                            }
                        ));
                    }
                }
            }
            result.push(r.join("\n"));
        }
        Content::Blob(b) => {
            let blob_info = blob_map
                .get(&path.to_string_lossy().to_string())
                .map_or(vec![], |m| m.get(&b.name).unwrap_or(&vec![]).to_vec());

            result.push(b.get_blame_header(&modifications, &blob_info));
            // result.push(format!("{} ({})", b.name, sha256::digest(&b.content)));
        }
    }

    result.join("\n")
}

pub fn generate_tree(tree: &Tree) -> String {
    return generate_subtree(&Content::Tree(tree.clone()));
}

fn generate_subtree(c: &Content) -> String {
    let mut result = vec![];

    match c {
        Content::Tree(t) => {
            let mut r = vec![t.name.clone()];
            if t.content.len() >= 1 {
                let length = t.content.len() - 1;
                for (index, i) in t.content.iter().enumerate() {
                    for (inner_index, line) in generate_subtree(i).split("\n").enumerate() {
                        r.push(format!(
                            " {} {line}",
                            if index == length {
                                if inner_index == 0 {
                                    "└"
                                } else {
                                    ""
                                }
                            } else {
                                if inner_index == 0 {
                                    "├"
                                } else {
                                    "│"
                                }
                            }
                        ));
                    }
                }
            }
            result.push(r.join("\n"));
        }
        Content::Blob(b) => {
            result.push(b.name.clone());
            // result.push(format!("{} ({})", b.name, sha256::digest(&b.content)));
        }
    }

    result.join("\n")
}

pub fn get_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards (???)")
        .as_millis() as u64
}

pub fn into_human_readable(t: u64) -> String {
    // accepts unix time, but only in milliseconds format
    DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_millis(t as u64))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}
