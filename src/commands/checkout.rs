use clap::ArgMatches;

use crate::core::{
    branch::branch::{Branch, BranchSource},
    error::{BranchError, RelicError},
    state::State,
};

pub fn checkout(state: Option<&mut State>, args: &ArgMatches) {
    let Some(state) = state else { return };

    let branch_name = args.get_one::<String>("BRANCH").unwrap().clone();
    let create_new = args.get_count("new") != 0;
    match Branch::set_head_branch(branch_name.clone(), state) {
        Some(err) => match err {
            RelicError::BranchError(BranchError::BranchDoesntExist) => {
                println!("Branch '{branch_name}' doesn't exist.");
                if create_new {
                    println!("Creating new branch ({branch_name})");
                    let r = state.fetch_local_head_commit();

                    if let Ok(Some(c)) = &r {
                        println!("Using {}", c.get_nickname());
                    }

                    match Branch::instantiate(
                        branch_name.clone(),
                        r.unwrap_or(None),
                        state,
                        &BranchSource::Local,
                    ) {
                        Ok(_) => match Branch::set_head_branch(branch_name.clone(), state) {
                            Some(e) => println!("Can't update branch: {e:?}"),
                            None => println!("Successfully changed branch to '{branch_name}'"),
                        },
                        Err(e) => {
                            println!("Can't create new branch: {e:?}");
                        }
                    }
                } else {
                    println!("Use the -n flag to create the branch.");
                }
            }
            e => println!("Error: {e:?}"),
        },
        None => println!("Successfully changed branch to '{branch_name}'"),
    }
}
