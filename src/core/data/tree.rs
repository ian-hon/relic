/*
Order by type (trees first) then filename (a-z)

Tree format:
T {byte length}\0
T {tree_name} hash
T {tree_name} hash
T {tree_name} hash
B {blob_name} hash
B {blob_name} hash
*/

use crate::core::data::{
    object::{Object, ObjectType},
    oid::ObjectID,
    util::oid_digest,
};

pub struct Tree {
    children: Vec<TreeEntry>,
}

impl Object for Tree {
    fn get_oid(&self) -> ObjectID {
        oid_digest("").into()
    }

    fn as_string(&self) -> String {
        self.children
            .iter()
            .map(|c| format!("{} {} {}", c.otype.to_string(), c.name, c.oid.to_string()))
            .fold(String::new(), |mut left, right| {
                left.reserve(right.len() + 1);
                left.push_str(&right);
                left.push_str("\n");
                left
            })
            .trim_end()
            .to_string() // EXPENSIVE!
    }
}

struct TreeEntry {
    oid: ObjectID,
    name: String,
    otype: ObjectType,
}
