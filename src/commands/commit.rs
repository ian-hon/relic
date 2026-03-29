use std::path::Path;

use crate::core::{
    branch::branch::{Branch, BranchSource, HeadType, DEFAULT_BRANCH},
    data::{commit::Commit, tree::Tree},
    error::{IOError, RelicError, RELIC_ERROR_CORRUPTED},
    modification::change::Change,
    object::ObjectLike,
    state::State,
    util::get_time,
};
use clap::Args;

#[derive(Args)]
pub struct CommitArgs {
    /// Commit message
    #[arg(short, long)]
    pub message: String,

    /// Commit description
    #[arg(short, long)]
    pub description: Option<String>,
}

pub fn commit(state: &mut State, args: CommitArgs) {
    let description = args.description.as_deref().unwrap_or("");

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

                    let c = Change::get_change_all(
                        &previous_commit
                            .tree
                            .construct_strict::<Tree>(&state.get_sanctum_path())
                            .unwrap(),
                        &tree,
                        &state.get_sanctum_path(),
                        &state.root_path,
                    );

                    println!("{}", c.as_human_readable(&tree, &state.get_sanctum_path()));

                    let c = Commit::new(
                        tree.get_oid(),
                        Some(previous_commit.get_oid()),
                        vec![],
                        get_time(),
                        "none".to_string(),
                        args.message.clone(),
                        description.to_string(),
                        &state.get_sanctum_path(),
                    );

                    if let Err(e) = Branch::update_branch(&b.name, &c, state, &BranchSource::Local)
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
                        args.message.clone(),
                        description.to_string(),
                        &state.get_sanctum_path(),
                    );

                    if let Err(e) = Branch::update_branch(&b.name, &c, state, &BranchSource::Local)
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
                    args.message,
                    description.to_string(),
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
