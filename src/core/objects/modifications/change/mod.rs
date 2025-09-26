mod constructor;
mod filter;
mod inverse;
mod serialisation;

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::core::modifications;

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
