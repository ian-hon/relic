use crate::core::modifications;

use super::Change;

impl Change {
    pub fn inverse(&self) -> Change {
        // TODO: test

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
