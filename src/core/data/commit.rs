use std::path::Path;

use crate::core::{
    object::ObjectLike,
    oid::ObjectID,
    util::{empty_oid, oid_digest, string_to_oid, url_decode, url_encode},
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

#[derive(Debug)]
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

    pub fn deserialise(payload: Vec<u8>) -> Option<Commit> {
        // TODO: test

        // takes payload and deserialises into Option<Commit>
        let payload = str::from_utf8(&payload).unwrap();

        let mut lines = payload.lines();
        lines.next(); // skip the header

        let tree = lines.next()?;
        let parent = lines.next()?;
        let timestamp = match (lines.next()?).parse::<u64>() {
            Ok(t) => t,
            Err(_) => return None,
        };
        let author = lines.next()?.to_string();
        let title = url_decode(lines.next()?);
        let description = url_decode(lines.next()?);

        /*
        tree {oid}
        parent {oid}
        timestamp
        author
        title
        description
        */

        let tree_elements = tree.split(" ").collect::<Vec<&str>>();
        if tree_elements.len() < 2 {
            return None;
        }
        let tree: ObjectID = string_to_oid(tree_elements[1]).into();

        let parent_elements = parent.split(" ").collect::<Vec<&str>>();
        let parent = match parent_elements.len() {
            2 => Some(string_to_oid(parent_elements[1]).into()),
            _ => None,
        };

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

        Some(c)
    }

    // #region actual logic
    pub fn get_state(upstream: ObjectID, local: ObjectID) -> CommitState {
        // dfs?
        // keep hashset of both commit's ids
        // upstream's hashset & local's hashset
        // traverse downwards from upstream and local once
        //  if hashset contains, then yada yada logic
        //  else append commit id to respective hashset

        CommitState::None
    }
    // #endregion
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

pub enum CommitState {
    Ahead, // local has commits that upstream doesnt
    /*
    Upstream: A > B > C
    Local   : A > B > C > D > E
    */
    Behind, // upstream has commits that local doesnt
    /*
    Upstream: A > B > C > D > E
    Local   : A > B > C
     */
    Tie, // both are equal
    /*
    Upstream: A > B > C
    Local   : A > B > C
     */
    Conflict, // upstream and local have conflicting commits
    /*
    Upstream: A > B > C > D > E
    Local   : A > B > C > F > G

    Two types of conflicts:
        Resolved
            There are no conflicts in the changes between upstream and local
            Basically, upstream and local did not modify any of the same files

            What to do:
                F.parent = E
                                    U   U   L   L
                Result: A > B > C > D > E > F > G

        Unresolved
            There are conflicts in the changes between upstream and local
            Upstream and local modified the same files
            Dont know whether to use upstream's or local's changes

            What to do:
                F.parent = E

                Create new commit (H) to resolve this
                User chooses which changes (in conflicted file) to apply

                                    U   U   L   L  Fix
                Result: A > B > C > D > E > F > G > H
    */
    None, // cant detect correlation between these commits
          /*
          Upstream: A > B > C
          Local   : X > Y > Z
          */
}
