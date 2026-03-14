use std::path::Path;

use clap::ArgMatches;

use crate::core::{
    branch::branch::{Branch, HeadType, DEFAULT_BRANCH},
    data::{commit::Commit, tree::Tree},
    error::{IOError, RelicError},
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
                    RelicError::ConfigurationIncorrect => "corrupted file",
                    RelicError::IOError(i) => match i {
                        IOError::FileNoExist | IOError::FileCantOpen => "head not found",
                        _ => "incorrect configuration",
                    },
                    _ => "incorrect configuration",
                }
            );
            return;
        }
    };

    // match state.
    if let Ok(head) = Branch::get_head(state) {
        match head {
            HeadType::Branch(b) => {
                // use head as parent
                if let Ok(previous_commit) = b.clone().get_commit(&state.get_sanctum_path()) {
                    // if previous_commit.tree == tree.get_oid() {
                    //     println!("no changes to commit");
                    //     return;
                    // }

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

                    println!(
                        "PREVIOUS:\n{}\n{}",
                        previous_commit.get_oid().to_string(),
                        str::from_utf8(&c.serialise()).unwrap()
                    );

                    println!("writing: {}", c.get_oid().to_string());

                    println!("{:?}", Branch::update_branch(b.name, c, state));
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

                    println!(
                        "branch exists, but no previous commit; writing new: {}",
                        c.get_oid().to_string()
                    );

                    println!("{:?}", Branch::update_branch(b.name, c, state));
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

                // println!("{:?}", Branch::update_branch("main".to_string(), c, state));
                println!(
                    "{:?}",
                    Branch::instantiate(DEFAULT_BRANCH.to_string(), Some(c), state)
                );
                println!("setting current head to new branch");
                Branch::set_head_branch(DEFAULT_BRANCH.to_string(), state);
            }
        }
    }

    // update local head only
    // match state.fetch_head_commit() {
    //     Ok(head) => {
    //         if let Some(head) = head {
    //             // use head as parent

    //             // if head.tree == tree.get_oid() {
    //             //     println!("no changes to commit");
    //             //     return;
    //             // }

    //             let c = Commit::new(
    //                 tree.get_oid(),
    //                 Some(head.get_oid()),
    //                 vec![],
    //                 get_time(),
    //                 "none".to_string(),
    //                 message,
    //                 description,
    //                 &state.get_sanctum_path(),
    //             );

    //             println!(
    //                 "IN COMMIT\n{}\n{}",
    //                 head.get_oid().to_string(),
    //                 str::from_utf8(&c.serialise()).unwrap()
    //             );

    //             println!("writing: {}", c.get_oid().to_string());

    //             if let Ok(HeadType::Branch(b)) = Branch::get_head(state) {
    //                 println!("{:?}", Branch::update_branch(b.name, c, state));
    //             }

    //             // let _ = fs::write(state.get_head_path(), c.get_oid().to_string());
    //         } else {
    //             // write into the file
    //             let c = Commit::new(
    //                 tree.get_oid(),
    //                 None,
    //                 vec![],
    //                 get_time(),
    //                 "none".to_string(),
    //                 message,
    //                 description,
    //                 &state.get_sanctum_path(),
    //             );

    //             println!("writing as new: {}", c.get_oid().to_string());

    //             if let Ok(HeadType::Branch(b)) = Branch::get_head(state) {
    //                 println!("{:?}", Branch::update_branch(b.name, c, state));
    //             }

    //             // let _ = fs::write(state.get_head_path(), c.get_oid().to_string());
    //         }
    //         println!("success");
    //     }
    //     Err(e) => match e {
    //         RelicError::ConfigurationIncorrect => println!("corrupted file"),
    //         RelicError::IOError(i) => match i {
    //             IOError::FileNoExist | IOError::FileCantOpen => println!("head not found"),
    //             _ => println!("incorrect configuration"),
    //         },
    //         _ => println!("incorrect configuration"),
    //     },
    // }
}
