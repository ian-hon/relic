use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

// use crate::core::{modifications, Blob, Content, Tree};
use crate::core::{
    data::tree::{Tree, TreeEntry},
    modification,
    object::Object,
};

impl TreeEntry {
    pub fn blob_blame_header(
        &self,
        modifications: &HashMap<String, bool>,
        blob_info: &Vec<modification::BlobOp>,
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
                        .filter(|b| match b.mod_op {
                            // modification::Blob::Create(_, _, _, _) => true,
                            modification::change::ModOp::Create => true,
                            _ => false,
                        })
                        .count(),
                    blob_info
                        .iter()
                        .filter(|b| match b.mod_op {
                            modification::change::ModOp::Delete => true,
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
    sanctum_path: &PathBuf,
    tree_map: &HashMap<String, HashSet<modification::TreeOp>>,
    blob_map: &HashMap<String, HashMap<String, Vec<modification::BlobOp>>>,
) -> String {
    return generate_blame_subtree(
        // &Object::Tree(tree.clone()),
        &TreeEntry {
            oid: tree.oid,
            name: "".to_string(),
            otype: crate::core::object::ObjectType::Tree,
        },
        &PathBuf::from("."),
        sanctum_path,
        tree_map,
        blob_map,
    );
}

pub fn generate_blame_subtree(
    entry: &TreeEntry,
    path: &PathBuf,
    sanctum_path: &PathBuf,
    tree_map: &HashMap<String, HashSet<modification::TreeOp>>,
    blob_map: &HashMap<String, HashMap<String, Vec<modification::BlobOp>>>,
) -> String {
    let mut result = vec![];

    let modifications =
        tree_map
            .get(&path.to_string_lossy().to_string())
            .map_or(HashMap::new(), |h| {
                h.into_iter()
                    // .map(|v| match v {
                    //     modification::Tree::CreateTree(_, n)
                    //     | modification::Tree::CreateBlob(_, n) => (n.to_string(), true),
                    //     modification::Tree::DeleteTree(_, n)
                    //     | modification::Tree::DeleteBlob(_, n) => (n.to_string(), false),
                    // })
                    .map(|v| {
                        (
                            v.name.to_string(),
                            v.mod_op.eq(&modification::change::ModOp::Create),
                        )
                    })
                    .collect::<HashMap<String, bool>>()
            });

    match entry.oid.construct(sanctum_path) {
        Ok(Object::Tree(t)) => {
            let name = entry.name.clone();
            let mut r = vec![name];
            if t.children.len() >= 1 {
                let length = t.children.len() - 1;
                for (index, i) in t.children.iter().enumerate() {
                    let mut p = path.clone();
                    if !entry.name.is_empty() {
                        p = path.join(entry.name.clone());
                    }
                    for (inner_index, line) in
                        generate_blame_subtree(i, &p, sanctum_path, tree_map, blob_map)
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
        Ok(Object::Blob(_)) => {
            let blob_info = blob_map
                .get(&path.to_string_lossy().to_string())
                .map_or(vec![], |m| m.get(&entry.name).unwrap_or(&vec![]).to_vec());

            result.push(entry.blob_blame_header(&modifications, &blob_info));
            // result.push(format!("{} ({})", b.name, sha256::digest(&b.content)));
        }
        _ => unimplemented!(),
    }

    result.join("\n")
}
