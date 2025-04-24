use std::{fs, path::Path};

use crate::state::State;

#[derive(Debug)]
pub struct Bones {
    // holds
    //      history.changes
    //      now.changes
    //      root
    //      upstream
    pub upstream: State
}
impl Bones {
    pub fn empty() -> Bones {
        Bones {
            upstream: State::empty()
        }
    }

    pub fn init(path: String) {
        // .bones/
        //      history.changes
        //      now.changes
        //      root
        //      upstream
    }

    pub fn load(path: &Path) -> Option<Bones> {
        let mut result = Bones::empty();

        result.upstream = State::deserialise_state(match fs::read_to_string(path.join(".bones/upstream")) {
            Ok(data) => { data },
            Err(_) => { return None; }
        })?;

        Some(result)
    }
}
