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
    pub tree: ObjectID,            // tree of the commit
    pub parent: Option<ObjectID>,  // commit before this one
    pub surrogates: Vec<ObjectID>, // commits used in a merge
    // length = 0
    //      no merge (default)
    // length = 1
    //      regular merge with one other commit
    // length >= 2
    //      octopus merge (more than one commit merged with this)
    pub timestamp: u64,
    pub author: String,      // lets assume author names follow a strict format
    pub message: String,     // url encoded when saved
    pub description: String, // url encoded when saved
}
impl Commit {
    pub fn new(
        tree: ObjectID,
        parent: Option<ObjectID>,
        surrogates: Vec<ObjectID>,
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
            surrogates,
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
parent {}
{}timestamp {}
author {}
message {}
description {}",
            self.tree.to_string(),
            if let Some(p) = self.parent {
                p.to_string()
            } else {
                ObjectID::empty().to_string()
            },
            // EXPENSIVE!
            self.surrogates
                .iter()
                .fold("".to_string(), |mut left, right| {
                    left.push_str(&format!("surrogate {}\n", right.to_string()));
                    left
                })
                .to_string(),
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
        // takes payload and deserialises into Option<Commit>
        let payload = Object::extract_body(&payload)?; // remove the header
        let payload = str::from_utf8(&payload).unwrap();

        let pairs = parse_kv_pair(payload, " ");

        let tree = ObjectID::from_string(&pairs.get("tree")?[0]);
        let parent = {
            let o = string_to_oid(&pairs.get("parent")?[0]);
            if o == empty_oid() {
                None
            } else {
                Some(o.into())
            }
        };
        let surrogates = if let Some(s) = pairs.get("surrogate") {
            s.iter()
                .map(|p| ObjectID::new(string_to_oid(p)))
                .collect::<Vec<ObjectID>>()
        } else {
            vec![]
        };
        let timestamp = match pairs.get("timestamp")?[0].parse::<u64>() {
            Ok(t) => t,
            Err(_) => return None,
        };
        // EXPENSIVE!
        let author = pairs.get("author")?[0].clone();
        let message = if let Some(m) = pairs.get("message") {
            url_decode(&m[0])
        } else {
            "".to_string()
        };
        let description = if let Some(d) = pairs.get("description") {
            url_decode(&d[0])
        } else {
            "".to_string()
        };

        let mut c = Commit {
            oid: empty_oid().into(),
            tree,
            parent,
            surrogates,
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

    pub fn get_ancestors(&self, sanctum_path: &Path) -> Vec<Commit> {
        // get parents + surrogate for one singular commit
        if let Some(parent) = self.parent {
            let mut r = match parent.construct(sanctum_path) {
                Ok(o) => match o {
                    Object::Commit(c) => vec![c],
                    _ => return vec![],
                },
                Err(_) => return vec![],
            };

            r.append(
                &mut self
                    .surrogates
                    .iter()
                    .map(|p| match p.construct(sanctum_path) {
                        Ok(o) => match o {
                            Object::Commit(c) => Some(c),
                            _ => None,
                        },
                        Err(_) => None,
                    })
                    .filter_map(|x| x)
                    .collect(),
            );

            return r;
        }
        vec![]
    }

    pub fn get_all_ancestors(&self, sanctum_path: &Path) -> Vec<Commit> {
        // get parents + surrogate of all ancestors of this commit
        let mut result = vec![self.clone()];

        let mut current = self.get_ancestors(sanctum_path);
        while !current.is_empty() {
            let p = current[0].clone();
            result.append(&mut current.clone());
            current = p.get_ancestors(sanctum_path);
        }

        result
    }

    pub fn get_all_parents(&self, sanctum_path: &Path) -> Vec<Commit> {
        let mut result = vec![self.clone()];

        let mut current = self.clone();
        while let Some(p) = current.get_parent(sanctum_path) {
            current = p.clone();
            result.push(p);
        }

        result
    }

    pub fn get_state(upstream: Commit, local: Commit, sanctum_path: &Path) -> CommitState {
        // only care about HEAD
        // if l.head is inside u_set => Behind
        // if u.head is inside l_set => Ahead
        // if neither => None OR Conflict
        //      find the last common commit between upstream and local
        // //      if none exists => None

        // i dont think we need to care about the surrogate parents
        // (emphasis on think)

        if upstream.get_oid() == local.get_oid() {
            return CommitState::Tie;
        }

        let mut u_all = upstream.get_all_parents(sanctum_path);
        let mut l_all = local.get_all_parents(sanctum_path);
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
