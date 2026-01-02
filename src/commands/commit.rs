use std::{fs, path::Path};

use clap::ArgMatches;

use crate::core::{
    data::{commit::Commit, tree::Tree},
    error::{IOError, RelicError},
    object::ObjectLike,
    state,
    util::get_time,
};

pub fn commit(path: &Path, relic_path: &Path, args: &ArgMatches) {
    let message = args.get_one::<String>("message").unwrap().clone();
    let description = args
        .get_one::<String>("description")
        .map_or("".to_string(), String::clone);

    // TODO: add upstream checking in here
    // upstream + its objects will be from relic-backend
    // for now, just worry about local head

    // something new here
    // something else

    // first commit: 161346246741b5cde9be083f888cd14b0be5cbb7dba78dcb57d04457eda74f1b
    // second commit's parent: 7fe4195cc6c8d7e69924e0a183509f541f2661a4883863f62d9f5b39d7ed96e1

    let tree = match Tree::build_tree(path, &relic_path.join("sanctum")) {
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

    let sanctum_path = relic_path.join("sanctum");
    let head_path = relic_path.join("head");

    // upstream
    // 65c8ea5db9c106aa702fb33cc1a631ae82881ed6db120e5abf5ae29172a14fc8
    // 54789394f94057b6d2a362fee4f61152c3864ee008930b279901b6e06471c327
    // a2454c23c117a4411463518c40a85188314f982552d44a3d33755ab9823660d6 <
    // 233f377dc5c2e7cde403a58c4f906cf348f8e17f14dd9a602e122ace34978b58
    // b565162264c6be6be2e62918cc67022ffd49f4953511dca56a4687714f20bf97

    // local
    // 65c8ea5db9c106aa702fb33cc1a631ae82881ed6db120e5abf5ae29172a14fc8
    // 54789394f94057b6d2a362fee4f61152c3864ee008930b279901b6e06471c327
    // a2454c23c117a4411463518c40a85188314f982552d44a3d33755ab9823660d6 <
    // 56d450fe16b5cadbe925b29903963e99fdc4768d9b90521b5d1dd9bb871e12f6
    // 3ff716050b04ab3a7dd4575d0f436018ad03df6718f0c57a52cd15f039069454

    // update local head only
    match state::fetch_head(relic_path) {
        Ok(head) => {
            if let Some(head) = head {
                // use head as parent

                if head.tree == tree.get_oid() {
                    println!("no changes to commit");
                    return;
                }

                let c = Commit::new(
                    tree.get_oid(),
                    Some(head.get_oid()),
                    get_time(),
                    "none".to_string(),
                    message,
                    description,
                    &sanctum_path,
                );

                println!(
                    "IN COMMIT\n{}\n{}",
                    head.get_oid().to_string(),
                    c.serialise()
                );

                println!("writing: {}", c.get_oid().to_string());

                let _ = fs::write(head_path, c.get_oid().to_string());
            } else {
                // write into the file
                let c = Commit::new(
                    tree.get_oid(),
                    None,
                    get_time(),
                    "none".to_string(),
                    message,
                    description,
                    &sanctum_path,
                );

                println!("writing: {}", c.get_oid().to_string());

                let _ = fs::write(head_path, c.get_oid().to_string());
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
