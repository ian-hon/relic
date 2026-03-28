use std::path::Path;

use clap::ArgMatches;

use crate::core::{
    branch::branch::{Branch, BranchSource, HeadType, DEFAULT_BRANCH},
    data::{commit::Commit, tree::Tree},
    error::{IOError, RelicError, RELIC_ERROR_CORRUPTED},
    object::ObjectLike,
    state::State,
    util::get_time,
};

pub fn commit(state: Option<&mut State>, args: &ArgMatches) {
    let Some(state) = state else { return };

    let message = args.get_one::<String>("message").unwrap().clone();
    let description = args
        .get_one::<String>("description")
        .map_or("".to_string(), String::clone);

    let tree = match Tree::build_tree(
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
                    RelicError::IOError(i) => match i {
                        IOError::FileNoExist | IOError::FileCantOpen => format!("HEAD not found."),
                        _ => format!("Incorrect configuration. {RELIC_ERROR_CORRUPTED}"),
                    },
                    _ => format!("Incorrect configuration. {RELIC_ERROR_CORRUPTED}"),
                }
            );
            return;
        }
    };

    // match state.
    if let Ok(head) = Branch::get_head(state, &BranchSource::Local) {
        match head {
            HeadType::Branch(b) => {
                // use head as parent
                if let Ok(previous_commit) = b.clone().get_commit(&state.get_sanctum_path()) {
                    if previous_commit.tree == tree.get_oid() {
                        println!("No changes to commit.");
                        return;
                    }

                    let c = Commit::new(
                        tree.get_oid(),
                        Some(previous_commit.get_oid()),
                        vec![],
                        get_time(),
                        "none".to_string(),
                        message,
                        description,
                        &state.get_sanctum_path(),
                    );

                    if let Err(e) =
                        Branch::update_branch(b.name.clone(), c, state, &BranchSource::Local)
                    {
                        println!(
                            "Something went wrong writing to branch \"{}\": {:?}",
                            b.name, e
                        );
                    }
                } else {
                    let c = Commit::new(
                        tree.get_oid(),
                        None,
                        vec![],
                        get_time(),
                        "none".to_string(),
                        message,
                        description,
                        &state.get_sanctum_path(),
                    );

                    if let Err(e) =
                        Branch::update_branch(b.name.clone(), c, state, &BranchSource::Local)
                    {
                        println!(
                            "Something went wrong writing to branch \"{}\": {:?}",
                            b.name, e
                        );
                    }
                }
            }
            HeadType::Detached(_) => {
                println!("Unable to commit. Currently in detached HEAD mode.\nTo make changes, please checkout a branch.")
            }
            HeadType::Empty => {
                let c = Commit::new(
                    tree.get_oid(),
                    None,
                    vec![],
                    get_time(),
                    "none".to_string(),
                    message,
                    description,
                    &state.get_sanctum_path(),
                );

                println!(
                    "head is empty (no thoughts); instantiating new branch with commit: {}",
                    c.get_oid().to_string()
                );

                let branch_name = DEFAULT_BRANCH;
                if let Err(e) = Branch::instantiate(
                    branch_name.to_string(),
                    Some(c),
                    state,
                    &BranchSource::Local,
                ) {
                    println!(
                        "Something went wrong creating branch \"{branch_name}\": {:?}",
                        e
                    );
                    return;
                }
                println!("HEAD now set to {branch_name}");
                Branch::set_head_branch(branch_name.to_string(), state);
            }
        }
    }
}
