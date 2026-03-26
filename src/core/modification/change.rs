use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::core::{
    data::{
        blob::Blob,
        tree::{self, Tree, TreeEntry},
    },
    modification::{self, blob::BlobOpInfo, utils, TreeOpInfo, TreeType},
    object::Object,
};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModOp {
    Create,
    Delete,
}
impl ModOp {
    pub fn get_notation(&self) -> &str {
        match self {
            ModOp::Create => "+",
            ModOp::Delete => "-",
        }
    }

    pub fn from_notation(n: &str) -> Option<ModOp> {
        match n {
            "+" => Some(ModOp::Create),
            "-" => Some(ModOp::Delete),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Change {
    pub trees: Vec<modification::TreeOp>,
    pub blobs: Vec<modification::BlobOp>,
}
impl Change {
    pub fn empty() -> Change {
        Change {
            trees: vec![],
            blobs: vec![],
        }
    }

    pub fn get_affected_blobs(&self) -> Vec<String> {
        let mut blobs = vec![];
        for (_, parent) in self.as_map().1 {
            blobs.append(&mut parent.iter().map(|f| f.0.to_string()).collect())
        }
        blobs
    }

    pub fn as_human_readable(&self, tree: &Tree, sanctum_path: &PathBuf) -> String {
        // To get the human readable output, both the change and the tree must be supplied
        /*
            {full change}

            repo_name
             ├ (+) saturn
             ├ (-) jupiter
             └ huh/mod.rs [+11, -52]

            x files affected, x additions, x deletions

            Changes need to be applied to tree, so that files
            being created are visible, and files deleted still show up.
            Eg:
                Changes: (+)saturn, (-)jupiter
                Tree will be children = {..., jupiter}
                After apply, will be  = {..., saturn, jupiter}
                Then, can format properly. If not, certain files and etc will not show up

                As a result, changes applied are filtered to include only ones that create, not delete
        */

        // // HashMap<String, HashSet<modifications::Tree>>,
        // // HashMap<String, HashMap<String, Vec<modifications::Blob>>>,

        let mut changes = self.clone();
        println!("{:?}", self.trees);
        changes.trees = changes
            .trees
            .clone()
            .into_iter()
            .filter(|t| match t.mod_op {
                ModOp::Delete => false,
                _ => true,
            })
            .collect::<Vec<modification::TreeOp>>();
        println!("{:?}", self.trees);

        let (tree_map, blob_map) = self.as_map();

        // let mut current_upstream = current_upstream.clone();
        // current_upstream.apply_changes(&changes);

        let affected_files = blob_map
            .iter()
            .map(|(_, v)| v.keys().count())
            .sum::<usize>();

        let addition = blob_map
            .iter()
            .map(|(_, v)| {
                v.iter()
                    .map(|(_, b)| {
                        b.iter()
                            .filter(|o| match o.mod_op {
                                ModOp::Create => true,
                                _ => false,
                            })
                            .count()
                    })
                    .sum::<usize>()
            })
            .sum::<usize>();

        let deletion = blob_map
            .iter()
            .map(|(_, v)| {
                v.iter()
                    .map(|(_, b)| {
                        b.iter()
                            .filter(|o| match o.mod_op {
                                ModOp::Delete => true,
                                _ => false,
                            })
                            .count()
                    })
                    .sum::<usize>()
            })
            .sum::<usize>();

        format!(
            "{}\n\n{}\n\n{affected_files} files affected, {} additions, {} deletions",
            self.serialise_changes(),
            utils::generate_blame_tree(&tree, sanctum_path, &tree_map, &blob_map),
            addition,
            deletion
        )
        // unimplemented!()
    }

    pub fn as_map(
        &self,
    ) -> (
        HashMap<String, HashSet<modification::TreeOp>>,
        HashMap<String, HashMap<String, Vec<modification::BlobOp>>>,
    ) {
        // tree_map: map<parent_directory, Vec<changes>>
        // blob_map: map<parent_directory, map<file_name, Vec<changes>>>

        let mut tree_map = HashMap::new();
        for tree_op in &self.trees {
            tree_map
                .entry(tree_op.info.parent.clone())
                .or_insert(HashSet::new())
                .insert(tree_op.clone());
        }

        let mut blob_map = HashMap::new();
        for blob_op in &self.blobs {
            blob_map
                .entry(blob_op.info.parent.clone())
                .or_insert(HashMap::new())
                .entry(blob_op.info.file.clone())
                .or_insert(vec![])
                .push(blob_op.clone());
        }

        (tree_map, blob_map)
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

        result.push("=".to_string()); // container and blob section separator

        let mut blob_sections = HashMap::new();
        for blob in &self.blobs {
            blob_sections
                .entry(blob.info.extract_path())
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

                if let (Some(tree_type), Some(mod_op)) = (
                    modification::tree::TreeType::from_notation(container),
                    ModOp::from_notation(species),
                ) {
                    result.trees.push(modification::TreeOp {
                        tree_type,
                        mod_op,
                        info: modification::tree::TreeOpInfo {
                            parent: urlencoding::decode(parent).unwrap().to_string(),
                            name: urlencoding::decode(name).unwrap().to_string(),
                        },
                    });
                } else {
                    return None;
                }
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
                            if let Some(mod_op) = ModOp::from_notation(species) {
                                result.blobs.push(modification::BlobOp {
                                    mod_op,
                                    info: BlobOpInfo {
                                        parent: decoded_path,
                                        file: decoded_name,
                                        line,
                                        text: content_text,
                                    },
                                });
                            } else {
                                return None;
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

    pub fn get_change(
        path: String,
        upstream_blob: &Blob,
        current_blob: &Blob,
        blob_name: &str,
    ) -> Vec<modification::BlobOp> {
        // https://blog.jcoglan.com/2017/02/15/the-myers-diff-algorithm-part-2/
        // for our change algorithm, we will be using myers diff algorithm
        // basically a shortest distance problem, with downwards, rightwards and diagonal directions as movement choices
        // (note that diagonal movements do not contribute towards the distance)

        if upstream_blob.oid == current_blob.oid {
            return vec![];
        }

        // similar does not handle newlines at eof well at all
        // this is the workaround for it
        let upstream = format!("{}\n", upstream_blob.get_body_as_string().unwrap());
        let current = format!("{}\n", current_blob.get_body_as_string().unwrap());

        let mut result = vec![];
        let diff = TextDiff::from_lines(&upstream, &current);

        for change in diff.iter_all_changes().filter_map(|c| match c.tag() {
            ChangeTag::Equal => None,
            _ => Some(c),
        }) {
            result.push(match change.tag() {
                // ChangeTag::Delete => modification::Blob::Delete(
                //     path.clone(),
                //     blob_name.to_string(),
                //     change.old_index().unwrap(),
                //     change.to_string().strip_suffix("\n").unwrap().to_string(),
                // ),
                // ChangeTag::Insert => modification::Blob::Create(
                //     path.clone(),
                //     blob_name.to_string(),
                //     change.new_index().unwrap(),
                //     change.to_string().strip_suffix("\n").unwrap().to_string(),
                // ),
                ChangeTag::Delete => modification::BlobOp::new(
                    ModOp::Delete,
                    modification::BlobOpInfo::new(
                        path.clone(),
                        blob_name.to_string(),
                        change.old_index().unwrap(),
                        change.to_string().strip_suffix("\n").unwrap().to_string(),
                    ),
                ),
                ChangeTag::Insert => modification::BlobOp::new(
                    ModOp::Create,
                    modification::BlobOpInfo::new(
                        path.clone(),
                        blob_name.to_string(),
                        change.new_index().unwrap(),
                        change.to_string().strip_suffix("\n").unwrap().to_string(),
                    ),
                ),
                _ => panic!("Unmatched change type: {}", change),
            })
        }

        result
    }

    pub fn get_change_all(
        upstream: &tree::Tree,
        current: &tree::Tree,
        sanctum_path: &PathBuf,
        path: &Path,
    ) -> Change {
        // assume that both current and previous have the same tree names
        // has to be bfs

        // initialise current & upstream state set
        let init_state_set = |children: &Vec<TreeEntry>| {
            let mut s = HashSet::new();
            let mut m = HashMap::new();
            for c in children {
                // TODO: use Object::extract_header here instead
                let f = match c.oid.construct(sanctum_path) {
                    Ok(Object::Tree(_)) => false,
                    Ok(Object::Blob(_)) => true,
                    _ => unimplemented!(),
                };

                s.insert((c.name.clone(), f));
                m.insert((c.name.clone(), f), c.clone());
            }
            (s, m)
        };
        // storing both the set and the map
        // set is just for performance
        let (current_set, current_map) = init_state_set(&current.children);
        let (upstream_set, upstream_map) = init_state_set(&upstream.children);
        //

        // use set differences to determine blob and tree creation or deletion
        // wonder if theres an easier to clone the values here
        let deleted = upstream_set
            .difference(&current_set)
            .map(|(n, t)| (n.to_string(), *t))
            .collect::<Vec<(String, bool)>>();
        let created = current_set
            .difference(&upstream_set)
            .map(|(n, t)| (n.to_string(), *t))
            .collect::<Vec<(String, bool)>>();
        //

        // for all deleted blobs, log them
        // for all deleted trees, log them and do the same for all children
        let mut tree_mods = vec![];
        let mut blob_mods = vec![];
        for (name, is_blob) in deleted {
            if is_blob {
                tree_mods.push(modification::TreeOp::new(
                    modification::TreeType::Blob,
                    ModOp::Delete,
                    TreeOpInfo::new(path.to_string_lossy().to_string(), name),
                ));
            } else {
                tree_mods.push(modification::TreeOp::new(
                    TreeType::Tree,
                    ModOp::Delete,
                    TreeOpInfo::new(path.to_string_lossy().to_string(), name.clone()),
                ));
                // traverse all children, add them to result as well
                let mut changes = Change::get_change_all(
                    &(match upstream_map
                        .get(&(name.clone(), false))
                        .unwrap()
                        .oid
                        .construct(&sanctum_path)
                    {
                        Ok(Object::Tree(deleted_tree)) => deleted_tree,
                        _ => panic!(),
                    }),
                    &Tree::empty(),
                    sanctum_path,
                    &path.join(name.clone()),
                );
                tree_mods.append(&mut changes.trees);
                blob_mods.append(&mut changes.blobs);
            }
        }
        //

        // for all created blobs, log them
        // for all created trees, log them and do the same for all children
        for (name, is_blob) in created {
            if is_blob {
                tree_mods.push(modification::TreeOp::new(
                    TreeType::Blob,
                    ModOp::Create,
                    TreeOpInfo::new(path.to_string_lossy().to_string(), name.clone()),
                ));
                blob_mods.append(&mut Change::get_change(
                    path.to_string_lossy().to_string(),
                    &Blob::empty(),
                    &(match current_map
                        .get(&(name.clone(), true))
                        .unwrap()
                        .oid
                        .construct(sanctum_path)
                    {
                        Ok(Object::Blob(b)) => b,
                        _ => panic!(),
                    }),
                    &(name.clone()),
                ))
            } else {
                tree_mods.push(modification::TreeOp::new(
                    TreeType::Tree,
                    ModOp::Create,
                    TreeOpInfo::new(path.to_string_lossy().to_string(), name.clone()),
                ));

                let mut changes = Change::get_change_all(
                    &Tree::empty(),
                    // &created_tree,
                    &(match current_map
                        .get(&(name.clone(), false))
                        .unwrap()
                        .oid
                        .construct(sanctum_path)
                    {
                        Ok(Object::Tree(t)) => t,
                        _ => panic!(),
                    }),
                    sanctum_path,
                    &path.join(name.clone()),
                );
                tree_mods.append(&mut changes.trees);
                blob_mods.append(&mut changes.blobs);
            }
        }

        for entry in &current.children {
            match entry.oid.construct(sanctum_path) {
                Ok(Object::Tree(tree)) => {
                    // get the matching upstream tree
                    // if it doesnt exist, that means the content is new and can be ignored
                    // we ignore it because we have already logged it in the section above
                    let p = path.join(entry.name.clone());
                    let upstream_tree = match upstream_map.get(&(entry.name.clone(), false)) {
                        Some(u) => match u.oid.construct(sanctum_path) {
                            Ok(Object::Tree(u_t)) => u_t,
                            _ => panic!(),
                        },
                        _ => {
                            continue;
                        }
                    };
                    //

                    let mut changes =
                        Change::get_change_all(&upstream_tree, &tree, sanctum_path, &p);
                    tree_mods.append(&mut changes.trees);
                    blob_mods.append(&mut changes.blobs);
                }
                Ok(Object::Blob(b)) => {
                    let upstream_blob = match upstream_map.get(&(entry.name.clone(), true)) {
                        Some(c) => match c.oid.construct(sanctum_path) {
                            Ok(Object::Blob(b)) => b,
                            _ => panic!(),
                        },
                        None => {
                            continue;
                        }
                    };

                    blob_mods.append(&mut Change::get_change(
                        path.to_string_lossy().to_string(),
                        &upstream_blob,
                        &b,
                        &entry.name,
                    ));
                }
                _ => unimplemented!(),
            }
        }

        Change {
            trees: tree_mods,
            blobs: blob_mods,
        }
    }
}
