mod constructor;
mod filter;
mod inverse;
mod serialisation;

use std::{
    collections::{HashMap, HashSet},
    thread::current,
};

use serde::{Deserialize, Serialize};

use crate::core::{content_set::ContentSet, modifications, utils, State, Tree};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Change {
    pub trees: Vec<modifications::Tree>,
    pub blobs: Vec<modifications::Blob>,
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

    pub fn as_human_readable(&self, current_upstream: &Tree) -> String {
        // HashMap<String, HashSet<modifications::Tree>>,
        // HashMap<String, HashMap<String, Vec<modifications::Blob>>>,

        let mut changes = self.clone();
        println!("{:?}", self.trees);
        changes.trees = changes
            .trees
            .clone()
            .into_iter()
            .filter(|t| match t {
                modifications::Tree::DeleteBlob(_, _) | modifications::Tree::DeleteTree(_, _) => {
                    false
                }
                _ => true,
            })
            .collect::<Vec<modifications::Tree>>();
        println!("{:?}", self.trees);

        let (tree_map, blob_map) = self.as_map();

        let mut current_upstream = current_upstream.clone();
        current_upstream.apply_changes(&changes);

        /*
            {full change}

            repo_name
             ├ (+) saturn
             ├ (-) jupiter
             └ huh/mod.rs [+11, -52]

            x files affected, x additions, x deletions
        */

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
                            .filter(|i| match i {
                                modifications::Blob::Create(_, _, _, _) => true,
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
                            .filter(|i| match i {
                                modifications::Blob::Delete(_, _, _, _) => true,
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
            utils::generate_blame_tree(&current_upstream, &tree_map, &blob_map),
            addition,
            deletion
        )
    }

    pub fn as_map(
        &self,
    ) -> (
        HashMap<String, HashSet<modifications::Tree>>,
        HashMap<String, HashMap<String, Vec<modifications::Blob>>>,
    ) {
        // tree_map: map<parent_directory, Vec<changes>>
        // blob_map: map<parent_directory, map<file_name, Vec<changes>>>

        let mut tree_map = HashMap::new();
        for tree_modification in &self.trees {
            let path = match tree_modification {
                modifications::Tree::CreateTree(path, _)
                | modifications::Tree::DeleteTree(path, _)
                | modifications::Tree::CreateBlob(path, _)
                | modifications::Tree::DeleteBlob(path, _) => path.clone(),
            };

            assert_eq!(path, tree_modification.extract_data().0);

            tree_map
                .entry(path)
                .or_insert(HashSet::new())
                .insert(tree_modification.clone());
        }

        let mut blob_map = HashMap::new();
        for blob_modification in &self.blobs {
            let (parent_directory, file_name) = match blob_modification {
                modifications::Blob::Create(path, name, _, _) => (path.clone(), name.clone()),
                modifications::Blob::Delete(path, name, _, _) => (path.clone(), name.clone()),
            };

            assert_eq!(
                (parent_directory.clone(), file_name.clone()),
                blob_modification.extract_path()
            );
            blob_map
                .entry(parent_directory)
                .or_insert(HashMap::new())
                .entry(file_name)
                .or_insert(vec![])
                .push(blob_modification.clone());
        }

        (tree_map, blob_map)
    }
}
