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
    // 022993d2ac22def1a5762d48d3c720f1a759db7b513f57b9f8dbd7c787edec5b
    // 30dfa7f84678f0fa126df059eb6ae55ec78fc4ac9340090c710bc0a52b5a4b1d <
    // dd29cfecbd3a03386acc3755ec6a2c259f6275c94f262d04041dbbd58c366663
    // 2acbbd187411c766cad84124a427d48b8a50be5c47146e106790c514f2251d77

    // local
    // 65c8ea5db9c106aa702fb33cc1a631ae82881ed6db120e5abf5ae29172a14fc8
    // 022993d2ac22def1a5762d48d3c720f1a759db7b513f57b9f8dbd7c787edec5b
    // 30dfa7f84678f0fa126df059eb6ae55ec78fc4ac9340090c710bc0a52b5a4b1d <
    // 759a2e5bab337ee627fd9c5fab0c4df88e5c1b188de52970a6c653928a0e1545
    // a32fe740a1f9d85e3e7120f43548ec9876a2d60c17368ea4ef7fc213b6269bd2

    // update local head only
    match state::fetch_head(relic_path) {
        Ok(head) => {
            if let Some(head) = head {
                // use head as parent

                // if head.tree == tree.get_oid() {
                //     println!("no changes to commit");
                //     return;
                // }

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
