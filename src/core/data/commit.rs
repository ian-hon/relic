use std::{collections::HashSet, path::Path};

use crate::core::{
    object::{Object, ObjectLike},
    oid::ObjectID,
    util::{
        empty_oid, into_human_readable, oid_digest, parse_kv_pair, string_to_oid, url_decode,
        url_encode,
    },
};

/*
Commit format:
tree {oid}
parent {oid}
timestmap {timestamp}
author {author}
message {message}
description {description}
*/

const DELIMITER: &str = "C\0";
const MESSAGE_TRUNC_LENGTH: usize = 40;

#[derive(Debug, Clone)]
pub struct Commit {
    pub oid: ObjectID,
    pub tree: ObjectID,         // tree of the commit
    pub parents: Vec<ObjectID>, // commit before this one
    // length = 0
    //      first ever commit in this repo
    // length = 1
    //      regular commit. parent is OID of previous commit
    // length >= 2
    //      merge commit. parent[0] is the original parent.
    //      parent[1..] are the other commits merged into this branch
    pub timestamp: u64,
    pub author: String,      // lets assume author names follow a strict format
    pub message: String,     // url encoded when saved
    pub description: String, // url encoded when saved
}
impl Commit {
    pub fn new(
        tree: ObjectID,
        parents: Vec<ObjectID>,
        timestamp: u64,
        author: String,
        message: String,
        description: String,

        sanctum_path: &Path,
    ) -> Commit {
        let mut c = Commit {
            oid: empty_oid().into(),
            tree,
            parents,
            timestamp,
            author,
            message,
            description,
        };

        c.oid = oid_digest(&c.serialise()).into();

        println!("NEW COMMIT : {}", c.oid.to_string());

        c.write(sanctum_path);

        c
    }

    pub fn as_payload(&self) -> String {
        format!("{DELIMITER}{}", self.as_string())
    }

    pub fn as_string(&self) -> String {
        // tree {oid}
        // parent {oid}
        // timestmap {timestamp}
        // author {author}
        // message {message}
        // description {description}
        format!(
            "tree {}
{}
timestamp {}
author {}
message {}
description {}",
            self.tree.to_string(),
            if self.parents.is_empty() {
                format!("parent {}", ObjectID::new(empty_oid()).to_string())
            } else {
                // EXPENSIVE!
                self.parents
                    .iter()
                    .fold("".to_string(), |mut left, right| {
                        left.push_str(&format!("parent {}\n", right.to_string()));
                        left
                    })
                    .trim_end()
                    .to_string()
            },
            self.timestamp,
            self.author,
            url_encode(&self.message),
            url_encode(&self.description)
        )
    }

    pub fn get_message_trunc(&self, padding: bool) -> String {
        // truncation
        // "lorem ipsum dolor sit a..."
        //  |--MESSAGE_TRUNC_LENGTH--|

        // padding = false
        // "lorem ipsum"

        // padding = true
        // "lorem ipsum         "

        let s = if self.message.len() <= MESSAGE_TRUNC_LENGTH {
            self.message.clone()
        } else {
            format!("{}...", &self.message[..(MESSAGE_TRUNC_LENGTH - 3)])
        };
        if padding {
            format!("{:<MESSAGE_TRUNC_LENGTH$}", s)
        } else {
            format!("{s}")
        }
    }

    pub fn get_nickname(&self) -> String {
        format!(
            "{} {} {}",
            self.get_oid().as_trunc(),
            self.get_message_trunc(true),
            into_human_readable(self.timestamp)
        )
    }

    pub fn deserialise(payload: Vec<u8>) -> Option<Commit> {
        // TODO: test

        // takes payload and deserialises into Option<Commit>
        let payload = Object::extract_body(&payload)?; // remove the header
        let payload = str::from_utf8(&payload).unwrap();

        let pairs = parse_kv_pair(payload, " ");

        println!("pairs: {pairs:?}");

        // TODO: test these [0]s
        let tree = ObjectID::from_string(&pairs.get("tree")?[0]);
        let parents = pairs
            .get("parent")?
            .iter()
            .map(|p| ObjectID::new(string_to_oid(p)))
            .collect::<Vec<ObjectID>>();
        let timestamp = match pairs.get("timestamp")?[0].parse::<u64>() {
            Ok(t) => t,
            Err(_) => return None,
        };
        // EXPENSIVE!
        let author = pairs.get("author")?[0].clone();
        let message = pairs.get("message")?[0].clone();
        let description = if let Some(d) = pairs.get("description") {
            d[0].to_string()
        } else {
            "".to_string()
        };

        let mut c = Commit {
            oid: empty_oid().into(),
            tree,
            parents,
            timestamp,
            author,
            message,
            description,
        };

        c.oid = oid_digest(&c.serialise()).into();

        Some(c)
    }

    // #region actual logic
    pub fn get_parents(&self, sanctum_path: &Path) -> Vec<Commit> {
        self.parents
            .iter()
            .map(|p| match p.construct(sanctum_path) {
                Ok(o) => match o {
                    Object::Commit(c) => Some(c),
                    _ => None,
                },
                Err(_) => None,
            })
            .filter_map(|x| x)
            .collect()
    }

    pub fn get_all_ancestors(&self, sanctum_path: &Path) -> Vec<Commit> {
        // take entire parent
        let mut result = vec![self.clone()];

        let mut current = self.get_parents(sanctum_path);
        while !current.is_empty() {
            let p = current[0].clone();
            result.append(&mut current.clone());
            current = p.get_parents(sanctum_path);
        }

        result
    }

    pub fn get_all_nuclear_ancestors(&self, sanctum_path: &Path) -> Vec<Commit> {
        // take only parent[0]
        let mut result = vec![self.clone()];

        let mut current = self.get_parents(sanctum_path);
        while !current.is_empty() {
            let p = current[0].clone();
            result.push(p.clone());
            current = p.get_parents(sanctum_path);
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
            // check:
            // u.parents contain l.oid
            // l.parents contain u.oid
            return CommitState::Tie;
        }

        let mut u_all = upstream.get_all_ancestors(sanctum_path);
        let mut l_all = local.get_all_ancestors(sanctum_path);
        // EXPENSIVE!
        u_all.reverse();
        l_all.reverse();

        let u_set: HashSet<[u8; 32]> = HashSet::from_iter(u_all.iter().map(|x| x.get_oid().0));
        let l_set: HashSet<[u8; 32]> = HashSet::from_iter(l_all.iter().map(|x| x.get_oid().0));

        if l_set.contains(&upstream.get_oid().0) {
            if let Some((_, i)) = Commit::get_last_common(&u_all, &l_all) {
                return CommitState::Ahead(l_all[(i + 1)..].to_vec());
            }
            panic!("no common found: Ahead");
        }

        if u_set.contains(&local.get_oid().0) {
            if let Some((_, i)) = Commit::get_last_common(&u_all, &l_all) {
                return CommitState::Behind(u_all[(i + 1)..].to_vec());
            }
            panic!("no common found: Behind");
        }

        if let Some((c, _)) = Commit::get_last_common(&u_all, &l_all) {
            return CommitState::Conflict(c.clone());
        }
        CommitState::None
    }

    fn get_last_common(a: &Vec<Commit>, b: &Vec<Commit>) -> Option<(Commit, usize)> {
        // for a and b, oldest commit to newest commit

        // TODO: test
        // can use binary search here to speed things up
        let mut previous = None;
        for index in 0..(a.len().min(b.len())) {
            if a[index].get_oid() != b[index].get_oid() {
                return previous;
            }
            previous = Some((a[index].clone(), index));
        }
        previous
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
    Ahead(Vec<Commit>), // local has commits that upstream doesnt
    /*
    Upstream: A > B > C
    Local   : A > B > C > D > E

    Value stored in the Vec (in order): [D, E]
    */
    Behind(Vec<Commit>), // upstream has commits that local doesnt
    /*
    Upstream: A > B > C > D > E
    Local   : A > B > C

    Value stored in the Vec (in order): [D, E]
     */
    Tie, // both are equal
    /*
    Upstream: A > B > C
    Local   : A > B > C
     */
    Conflict(Commit), // upstream and local have conflicting commits
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
