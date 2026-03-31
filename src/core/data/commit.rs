use std::path::Path;

use crate::core::{
    object::{Object, ObjectLike, ObjectType},
    oid::ObjectID,
    util::{
        empty_oid, into_human_readable, oid_digest_data, parse_kv_pair, string_to_oid, url_decode,
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

// use pubkey as the uniq id
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

        c.oid = oid_digest_data(&c.serialise()).into();

        c.write(sanctum_path);

        c
    }

    pub fn as_payload(&self) -> Vec<u8> {
        format!("{DELIMITER}{}", self.as_string())
            .as_bytes()
            .to_vec()
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

    pub fn get_nickname(&self, padding: bool) -> String {
        format!(
            "({}) \"{}\" ({})",
            self.get_oid().as_trunc(),
            self.get_message_trunc(padding),
            into_human_readable(self.timestamp)
        )
    }

    pub fn deserialise(payload: Vec<u8>) -> Option<Commit> {
        // takes payload and deserialises into Option<Commit>
        let payload = Object::extract_body(&payload)?; // remove the header
        let payload = str::from_utf8(&payload).unwrap();

        let pairs = parse_kv_pair(payload, " ");

        let tree = ObjectID::from_string(&pairs.get("tree")?[0])?;
        let parent = {
            let o = string_to_oid(&pairs.get("parent")?[0])?;
            if o == empty_oid() {
                None
            } else {
                Some(o.into())
            }
        };
        let surrogates = if let Some(s) = pairs.get("surrogate") {
            let r = s.iter().map(|p| ObjectID::from_string(p));
            if r.clone().all(|i| i.is_some()) {
                r.map(|i| i.unwrap()).collect()
            } else {
                return None;
            }
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

        c.oid = oid_digest_data(&c.serialise()).into();

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
    // #endregion
}

impl ObjectLike for Commit {
    const OBJECT_TYPE: ObjectType = ObjectType::Commit;

    fn get_oid(&self) -> ObjectID {
        self.oid
    }

    fn as_string(&self) -> String {
        // returns without header
        self.as_string()
    }

    fn serialise(&self) -> Vec<u8> {
        // returns with header
        self.as_payload()
    }
}
