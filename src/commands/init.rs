use std::path::PathBuf;

use clap::Args;

use crate::core::{
    branch::branch::{Branch, BranchSource, DEFAULT_BRANCH},
    state::State,
};

#[derive(Args)]
pub struct InitArgs {}

pub fn init(path: &PathBuf, _args: InitArgs) {
    let Some(state) = State::initialise(path) else {
        println!("Unable to initialise a Relic repository.");
        return;
    };

    // make main branch
    if let Err(e) = Branch::instantiate(
        DEFAULT_BRANCH.to_string(),
        None,
        &state,
        &BranchSource::Local,
    ) {
        println!("Error creating branch: {e:?}");
        return;
    };

    if let Some(e) = Branch::set_head_branch(DEFAULT_BRANCH.to_string(), &state) {
        println!("Error setting HEAD: {e:?}");
    }

    println!("Relic repository initialised successfully.");
}
