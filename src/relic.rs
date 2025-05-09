use std::{fs, path::Path};

use crate::state::State;

#[derive(Debug)]
pub struct Relic {
    // holds
    //      history.changes
    //      now.changes
    //      root
    //      upstream
    pub upstream: State
}
impl Relic {
    pub fn empty() -> Relic {
        Relic {
            upstream: State::empty()
        }
    }

    pub fn init(path: String) {
        // .relic/
        //      history.changes
        //      now.changes
        //      root
        //      upstream
    }

    pub fn load(path: &Path) -> Option<Relic> {
        let mut result = Relic::empty();

        result.upstream = State::deserialise_state(match fs::read_to_string(path.join(".relic/upstream")) {
            Ok(data) => { data },
            Err(_) => { return None; }
        })?;

        Some(result)
    }
}
