use std::path::Path;

use clap::ArgMatches;

use crate::core::{
    data::commit::{Commit, CommitState},
    state::{fetch_head, fetch_upstream},
};

pub fn status(_: &Path, relic_path: &Path, _: &ArgMatches) {
    let sanctum_path = relic_path.join("sanctum");
    if !sanctum_path.exists() {
        println!("sanctum doesnt exist. is your relic configuration corrupted?");
        return;
    }

    // compare head and upstream
    match (fetch_head(relic_path), fetch_upstream(relic_path)) {
        (Ok(h), Ok(u)) => match (h, u) {
            (None, None) | (None, Some(_)) | (Some(_), None) => {
                println!("no pending commits");
                return;
            }
            (Some(head), Some(upstream)) => {
                match Commit::get_state(upstream, head, &sanctum_path) {
                    CommitState::Ahead(v) => {
                        println!("local is ahead by {} commits", v.len());
                        for c in v {
                            println!("{}", c.get_nickname());
                        }
                    }
                    CommitState::Behind(v) => {
                        println!("local is behind by {} commits", v.len());
                        for c in v {
                            println!("{}", c.get_nickname());
                        }
                    }
                    CommitState::Tie => {
                        println!("local is up to date with upstream");
                    }
                    CommitState::Conflict(ancestor) => {
                        println!(
                            "conflict between local and upstream. last common ancestor:\n{}",
                            ancestor.get_nickname()
                        );
                    }
                    CommitState::None => {
                        println!("upstream and local are not related. is your relic configuration corrupted?");
                    }
                }
            }
        },
        _ => {
            println!("cant seem to get upstream or head. is your relic configuration corrupted?")
        }
    }
}
