use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use similar::{ChangeTag, TextDiff};

use crate::core::{modifications, Blob, Content, Tree};

use super::Change;

impl Change {
    pub fn get_change(
        path: String,
        upstream_blob: &Blob,
        current_blob: &Blob,
    ) -> Vec<modifications::Blob> {
        // https://blog.jcoglan.com/2017/02/15/the-myers-diff-algorithm-part-2/
        // for our change algorithm, we will be using myers diff algorithm
        // basically a shortest distance problem, with downwards, rightwards and diagonal directions as movement choices
        // (note that diagonal movements do not contribute towards the distance)

        // similar does not handle newlines at eof well at all
        // this is the workaround for it
        let upstream = format!("{}\n", upstream_blob.content.clone());
        let current = format!("{}\n", current_blob.content.clone());

        // TODO : compare hashes instead of blobs
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
                    current_blob.name.clone(),
                    change.old_index().unwrap(),
                    change.to_string().strip_suffix("\n").unwrap().to_string(),
                ),
                ChangeTag::Insert => modifications::Blob::Create(
                    path.clone(),
                    current_blob.name.clone(),
                    change.new_index().unwrap(),
                    change.to_string().strip_suffix("\n").unwrap().to_string(),
                ),
                _ => panic!("Unmatched change type: {}", change),
            })
        }

        result
    }

    pub fn get_change_all(upstream: &Tree, current: &Tree, path: &Path) -> Change {
        // assume that both current and previous have the same tree names
        // has to be bfs

        // initialise current state set
        let mut current_set = HashSet::new();
        let mut current_map = HashMap::new();
        for c in &current.content {
            match c {
                Content::Tree(t) => {
                    current_set.insert((t.name.clone(), false));
                    current_map.insert((t.name.clone(), false), c);
                }
                Content::Blob(b) => {
                    current_set.insert((b.name.clone(), true));
                    current_map.insert((b.name.clone(), true), c);
                }
            }
        }
        //

        // initialise upstream state set
        let mut upstream_set = HashSet::new();
        let mut upstream_map = HashMap::new();
        for c in &upstream.content {
            match c {
                Content::Tree(t) => {
                    upstream_set.insert((t.name.clone(), false));
                    upstream_map.insert((t.name.clone(), false), c);
                }
                Content::Blob(b) => {
                    upstream_set.insert((b.name.clone(), true));
                    upstream_map.insert((b.name.clone(), true), c);
                }
            }
        }
        //

        // use set differences to determine blob and tree creation or deletion
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
        let mut container_modifications = vec![];
        let mut modifications = vec![];
        for (name, is_blob) in deleted {
            if is_blob {
                container_modifications.push(modifications::Tree::DeleteBlob(
                    path.to_string_lossy().to_string(),
                    name,
                ));
            } else {
                container_modifications.push(modifications::Tree::DeleteTree(
                    path.to_string_lossy().to_string(),
                    name.clone(),
                ));
                // traverse all children, add them to result as well
                let mut changes = Change::get_change_all(
                    match upstream_map.get(&(name.clone(), false)).unwrap() {
                        Content::Tree(deleted_tree) => deleted_tree,
                        _ => panic!(),
                    },
                    &Tree::new(),
                    &path.join(name.clone()),
                );
                container_modifications.append(&mut changes.trees);
                modifications.append(&mut changes.blobs);
            }
        }
        //

        // for all created blobs, log them
        // for all created trees, log them and do the same for all children
        for (name, is_blob) in created {
            if is_blob {
                container_modifications.push(modifications::Tree::CreateBlob(
                    path.to_string_lossy().to_string(),
                    name.clone(),
                ));
                modifications.append(&mut Change::get_change(
                    path.to_string_lossy().to_string(),
                    &Blob::new(),
                    match current_map.get(&(name, true)).unwrap() {
                        Content::Blob(b) => b,
                        _ => panic!(),
                    },
                ))
            } else {
                container_modifications.push(modifications::Tree::CreateTree(
                    path.to_string_lossy().to_string(),
                    name.clone(),
                ));

                let mut changes = Change::get_change_all(
                    &Tree::new(),
                    match current_map.get(&(name.clone(), false)).unwrap() {
                        Content::Tree(t) => t,
                        _ => panic!(),
                    },
                    &path.join(name.clone()),
                );
                container_modifications.append(&mut changes.trees);
                modifications.append(&mut changes.blobs);
            }
        }

        for content in &current.content {
            match content {
                Content::Tree(tree) => {
                    // get the matching upstream tree
                    // if it doesnt exist, that means the content is new and can be ignored
                    // we ignore it because we have already logged it in the section above
                    let p = path.join(tree.name.clone());
                    let upstream_tree = match upstream_map.get(&(tree.name.clone(), false)) {
                        Some(u) => match u {
                            Content::Tree(u_t) => u_t,
                            _ => panic!(),
                        },
                        _ => {
                            continue;
                        }
                    };
                    //

                    let mut changes = Change::get_change_all(upstream_tree, tree, &p);
                    container_modifications.append(&mut changes.trees);
                    modifications.append(&mut changes.blobs);
                }
                Content::Blob(b) => {
                    let upstream_blob = match upstream_map.get(&(b.name.clone(), true)) {
                        Some(c) => match c {
                            Content::Blob(b) => b,
                            _ => panic!(),
                        },
                        None => {
                            continue;
                        }
                    };

                    modifications.append(&mut Change::get_change(
                        path.to_string_lossy().to_string(),
                        &upstream_blob,
                        &b,
                    ));
                }
            }
        }

        Change {
            trees: container_modifications,
            blobs: modifications,
        }
    }
}
