use std::collections::HashMap;

use crate::core::modifications;

use super::Change;

impl Change {
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

        result.push("=".to_string()); // container and blob section separator

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
                        println!("invalid blob header");
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
}
