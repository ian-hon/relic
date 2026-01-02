use std::{collections::HashSet, path::Path};

use crate::core::{
    object::{Object, ObjectLike},
    oid::ObjectID,
    util::{empty_oid, oid_digest, string_to_oid, url_decode, url_encode},
};

/*
Commit format:
tree {oid}
parent {oid}
timestamp
author
message
description
*/

const DELIMITER: &str = "C\0";

#[derive(Debug, Clone)]
pub struct Commit {
    pub oid: ObjectID,
    pub tree: ObjectID,           // tree of the commit
    pub parent: Option<ObjectID>, // commit before this one
    pub timestamp: u64,
    pub author: String,      // lets assume author names follow a strict format
    pub message: String,     // url encoded when saved
    pub description: String, // url encoded when saved
}
impl Commit {
    pub fn new(
        tree: ObjectID,
        parent: Option<ObjectID>,
        timestamp: u64,
        author: String,
        message: String,
        description: String,

        sanctum_path: &Path,
    ) -> Commit {
        let mut c = Commit {
            oid: empty_oid().into(),
            tree,
            parent,
            timestamp,
            author,
            message,
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
        // message
        // description
        format!(
            "tree {}\nparent {}\n{}\n{}\n{}\n{}",
            self.tree.to_string(),
            self.parent.map_or_else(
                || Into::<ObjectID>::into(empty_oid()).to_string(),
                |p| p.to_string()
            ),
            self.timestamp,
            self.author,
            url_encode(&self.message),
            url_encode(&self.description)
        )
    }

    pub fn deserialise(payload: Vec<u8>) -> Option<Commit> {
        // TODO: test

        // takes payload and deserialises into Option<Commit>
        let payload = Object::extract_body(&payload)?; // remove the header
        let payload = str::from_utf8(&payload).unwrap();

        let mut lines = payload.lines();

        let tree = lines.next()?;
        let parent = lines.next()?;
        let l = lines.next()?;
        let timestamp = match (l).parse::<u64>() {
            Ok(t) => t,
            Err(_) => return None,
        };
        let author = lines.next()?.to_string();
        let message = url_decode(lines.next()?);
        let description = url_decode(lines.next().unwrap_or(""));

        /*
        tree {oid}
        parent {oid}
        timestamp
        author
        message
        description
        */

        let tree_elements = tree.split(" ").collect::<Vec<&str>>();
        if tree_elements.len() < 2 {
            return None;
        }
        let tree: ObjectID = string_to_oid(tree_elements[1]).into();

        let parent_elements = parent.split(" ").collect::<Vec<&str>>();
        if parent_elements.len() < 2 {
            return None;
        }
        let parent: ObjectID = Into::<ObjectID>::into(string_to_oid(parent_elements[1]));
        // if theres a collision with 64 0s then ill be super happy
        let parent = if parent == empty_oid().into() {
            None
        } else {
            Some(parent)
        };

        let mut c = Commit {
            oid: empty_oid().into(),
            tree,
            parent,
            timestamp,
            author,
            message,
            description,
        };

        c.oid = oid_digest(&c.serialise()).into();

        Some(c)
    }

    // #region actual logic
    pub fn get_parent(&self, sanctum_path: &Path) -> Option<Commit> {
        match self.parent {
            Some(p) => p.construct(&sanctum_path).map_or_else(
                |_| None,
                |p| match p {
                    Object::Commit(c) => Some(c),
                    _ => None,
                },
            ),
            None => None,
        }
    }

    pub fn get_all_previous(&self, sanctum_path: &Path) -> Vec<Commit> {
        let mut result = vec![self.clone()];

        let mut current = self.clone();
        while let Some(p) = current.get_parent(sanctum_path) {
            current = p.clone();
            result.push(p);
        }

        result
    }

    pub fn get_state(upstream: Commit, local: Commit, sanctum_path: &Path) -> CommitState {
        // dfs?
        // keep hashset of both commit's ids
        // upstream's hashset & local's hashset
        // traverse downwards from upstream and local once
        //  if hashset contains, then yada yada logic
        //  else append commit id to respective hashset

        // wrong logic
        // only care about HEAD
        // if l.head is inside u_set => Behind
        // if u.head is inside l_set => Ahead
        // if neither => None OR Conflict
        //      find the last common commit between upstream and local
        // //      if none exists => None

        if upstream.get_oid() == local.get_oid() {
            return CommitState::Tie;
        }

        let u_all = upstream.get_all_previous(sanctum_path);
        let l_all = local.get_all_previous(sanctum_path);

        let u_set: HashSet<[u8; 32]> = HashSet::from_iter(u_all.iter().map(|x| x.get_oid().0));
        let l_set: HashSet<[u8; 32]> = HashSet::from_iter(l_all.iter().map(|x| x.get_oid().0));

        if l_set.contains(&upstream.get_oid().0) {
            return CommitState::Ahead;
        }

        if u_set.contains(&local.get_oid().0) {
            return CommitState::Behind;
        }

        // can use binary search here to speed things up
        for index in 0..(u_all.len().min(l_all.len())) {
            if u_all[index].get_oid() == l_all[index].get_oid() {
                return CommitState::Conflict(u_all[index].get_oid());
            }
        }

        CommitState::None

        // if upstream.get_oid() == local.get_oid() {
        //     return CommitState::Tie;
        // }

        // let mut u_set: HashSet<[u8; 32]> = HashSet::new();
        // let mut l_set: HashSet<[u8; 32]> = HashSet::new();

        // u_set.insert(upstream.get_oid().0);
        // l_set.insert(local.get_oid().0);

        // // // if upstream != local and they dont have parents, its CommitState::None
        // // let mut current_upstream = match upstream.get_parent(sanctum_path) {
        // //     Some(u) => u,
        // //     None => return CommitState::None,
        // // };
        // // let mut current_local = match local.get_parent(sanctum_path) {
        // //     Some(l) => l,
        // //     None => return CommitState::None,
        // // };

        // let mut current_upstream = upstream.clone();
        // let mut current_local = local.clone();

        // let mut upstream_reached_end = false;
        // let mut local_reached_end = false;

        // while !upstream_reached_end && !local_reached_end {
        //     if !upstream_reached_end {
        //         match current_upstream.get_parent(sanctum_path) {
        //             Some(u) => {
        //                 u_set.insert(u.get_oid().0);
        //                 current_upstream = u;
        //             }
        //             None => upstream_reached_end = true,
        //         };
        //     }

        //     if !local_reached_end {
        //         match current_local.get_parent(sanctum_path) {
        //             Some(l) => {
        //                 l_set.insert(l.get_oid().0);
        //                 current_local = l;
        //             }
        //             None => local_reached_end = true,
        //         };
        //     }

        //     // check if current_upstream is inside l_set
        //     // check if current_local is inside u_set
        //     let u_found = l_set.contains(&current_upstream.get_oid().0);
        //     let l_found = u_set.contains(&current_local.get_oid().0);

        //     match (u_found, l_found) {
        //         (false, false) => {
        //             // do nothing and continue
        //         }
        //         (false, true) => {
        //             return CommitState::Behind;
        //         }
        //         (true, false) => {
        //             return CommitState::Ahead;
        //         }
        //         (true, true) => {}
        //     }
        // }

        // CommitState::None
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

#[derive(Debug)]
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
    Conflict(ObjectID), // upstream and local have conflicting commits
    // Conflict({last common commit})
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
