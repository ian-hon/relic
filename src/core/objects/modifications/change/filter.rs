use std::path::PathBuf;

use crate::core::{content_set::ContentSet, modifications};

use super::Change;

impl Change {
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
                    // if only can map a tuple
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
}
