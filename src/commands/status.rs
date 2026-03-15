use clap::ArgMatches;

use crate::core::{
    data::commit::{Commit, CommitState},
    state::State,
};

pub fn status(state: Option<&mut State>, _: &ArgMatches) {
    let Some(state) = state else { return };

    // compare head and upstream
    // match (fetch_head(relic_path), fetch_upstream(relic_path)) {
    match (
        state.fetch_local_head_commit(),
        state.fetch_upstream_head_commit(),
    ) {
        (Ok(h), Ok(u)) => match (h, u) {
            (None, None) | (None, Some(_)) | (Some(_), None) => {
                println!("No pending commits.");
                return;
            }
            (Some(head), Some(upstream)) => {
                match Commit::get_state(upstream, head, &state.get_sanctum_path()) {
                    CommitState::Ahead(v) => {
                        println!("Local is ahead by {} commits.", v.len());
                        for c in v {
                            println!("{}", c.get_nickname());
                        }
                    }
                    CommitState::Behind(v) => {
                        println!("Local is behind by {} commits.", v.len());
                        for c in v {
                            println!("{}", c.get_nickname());
                        }
                    }
                    CommitState::Tie => {
                        println!("Local is up to date with upstream.");
                    }
                    CommitState::Conflict(ancestor) => {
                        println!(
                            "Divergence between local and upstream. Last common ancestor:\n{}",
                            ancestor.get_nickname()
                        );
                    }
                    CommitState::None => {
                        println!("Upstream and local are not related. Is your relic configuration corrupted?");
                    }
                }
            }
        },
        _ => {
            println!(
                "Cant seem to get either upstream or head. Is your relic configuration corrupted?"
            )
        }
    }
}
