use std::path::Path;

use crate::core::{
    object::ObjectLike,
    oid::ObjectID,
    util::{empty_oid, oid_digest, url_encode},
};

/*
Commit format:
tree {oid}
parent {oid}
timestamp
author
title
description
*/

const DELIMITER: &str = "C\0";

pub struct Commit {
    pub oid: ObjectID,
    pub tree: ObjectID,           // tree of the commit
    pub parent: Option<ObjectID>, // commit before this one
    pub timestamp: u64,
    pub author: String,      // lets assume author names follow a strict format
    pub title: String,       // url encoded when saved
    pub description: String, // url encoded when saved
}
impl Commit {
    pub fn new(
        tree: ObjectID,
        parent: Option<ObjectID>,
        timestamp: u64,
        author: String,
        title: String,
        description: String,

        sanctum_path: &Path,
    ) -> Commit {
        let mut c = Commit {
            oid: empty_oid().into(),
            tree,
            parent,
            timestamp,
            author,
            title,
            description,
        };

        c.oid = oid_digest(&c.serialise()).into();

        c.write(sanctum_path);

        c
    }

    pub fn as_payload(&self) -> String {
        format!("{DELIMITER}{}", self.as_string())
    }

    pub fn as_string(&self) -> String {
        // tree {oid}
        // parent {oid}
        // timestamp
        // author
        // title
        // description
        format!(
            "tree {}\nparent {}\n{}\n{}\n{}\n{}",
            self.tree.to_string(),
            self.parent
                .map_or_else(|| "".to_string(), |p| p.to_string()),
            self.timestamp,
            self.author,
            url_encode(&self.title),
            url_encode(&self.description)
        )
    }
}

impl ObjectLike for Commit {
    fn get_oid(&self) -> ObjectID {
        self.oid
    }

    fn as_string(&self) -> String {
        // returns without header
        self.as_string()
    }

    fn serialise(&self) -> String {
        // returns with header
        self.as_payload()
    }
}
