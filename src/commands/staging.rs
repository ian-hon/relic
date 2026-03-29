use std::path::Path;

use crate::core::{
    branch::branch::{Branch, BranchSource},
    data::tree::Tree,
    error::{IOError, RelicError, RELIC_ERROR_CORRUPTED},
    modification::change::Change,
    state::State,
};
use clap::ArgMatches;

pub fn staging(state: Option<&mut State>, _: &ArgMatches) {
    let Some(state) = state else { return };

    let local = match Tree::build_tree(
        &state,
        &state.root_path,
        &state.get_sanctum_path(),
        Path::new("."),
    ) {
        Ok(t) => t,
        Err(e) => {
            println!(
                "{}",
                match e {
                    RelicError::ConfigurationIncorrect =>
                        format!("Can't build tree. {RELIC_ERROR_CORRUPTED}"),
                    RelicError::IOError(_) => format!("Tree can't be built: {e:?}"),
                    _ => format!("Incorrect configuration. {RELIC_ERROR_CORRUPTED}"),
                }
            );
            return;
        }
    };

    let upstream = Branch::get_head(state, &BranchSource::Local)
        .ok()
        .and_then(|h| h.get_commit(&state.get_sanctum_path()).ok().flatten())
        .and_then(|c| c.tree.construct_strict::<Tree>(&state.get_sanctum_path()));

    let Some(upstream) = upstream else {
        println!("Unable to fetch HEAD.");
        return;
    };

    println!(
        "{}",
        Change::get_change_all(
            &upstream,
            &local,
            &state.get_sanctum_path(),
            &state.root_path,
        )
        .as_human_readable(&local, &state.get_sanctum_path())
    );
    //
}
