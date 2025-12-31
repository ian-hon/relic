use crate::core::data::util::{empty_oid, oid_to_string};

#[derive(Clone, Copy, Debug)]
pub struct ObjectID([u8; 32]);
impl ObjectID {
    pub fn new(oid: [u8; 32]) -> ObjectID {
        ObjectID(oid)
    }

    pub fn empty() -> ObjectID {
        ObjectID(empty_oid())
    }
}

impl Into<ObjectID> for [u8; 32] {
    fn into(self) -> ObjectID {
        ObjectID(self)
    }
}

impl ToString for ObjectID {
    fn to_string(&self) -> String {
        oid_to_string(self.0)
    }
}
