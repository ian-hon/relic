use std::{fs, path::Path};

use clap::ArgMatches;

use crate::core::{
    data::{commit::Commit, tree::Tree},
    error::{IOError, RelicError},
    object::ObjectLike,
    state::State,
    util::{get_time, string_to_oid},
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

    // update local head only
    match state.fetch_head() {
        Ok(head) => {
            if let Some(head) = head {
                // use head as parent

                // if head.tree == tree.get_oid() {
                //     println!("no changes to commit");
                //     return;
                // }

                let c = Commit::new(
                    tree.get_oid(),
                    // vec![head.get_oid()],
                    vec![
                        head.get_oid(),
                        string_to_oid(
                            "839850acdfa832ab1125d22d4eb936ed317a9dca637ca19d4c96dd21143cecc5",
                        )
                        .into(),
                    ],
                    get_time(),
                    "none".to_string(),
                    message,
                    description,
                    &state.get_sanctum_path(),
                );

                println!(
                    "IN COMMIT\n{}\n{}",
                    head.get_oid().to_string(),
                    c.serialise()
                );

                println!("writing: {}", c.get_oid().to_string());

                let _ = fs::write(state.get_head_path(), c.get_oid().to_string());
            } else {
                // write into the file
                let c = Commit::new(
                    tree.get_oid(),
                    vec![],
                    get_time(),
                    "none".to_string(),
                    message,
                    description,
                    &state.get_sanctum_path(),
                );

                println!("writing: {}", c.get_oid().to_string());

                let _ = fs::write(state.get_head_path(), c.get_oid().to_string());
            }
            println!("success");
        }
        Err(e) => match e {
            RelicError::ConfigurationIncorrect => println!("corrupted file"),
            RelicError::IOError(i) => match i {
                IOError::FileNoExist | IOError::FileCantOpen => println!("head not found"),
                _ => println!("incorrect configuration"),
            },
            _ => println!("incorrect configuration"),
        },
    }
}
