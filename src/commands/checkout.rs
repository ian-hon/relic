use clap::ArgMatches;

use crate::core::{
    branch::branch::{Branch, BranchSource, HeadType},
    data::commit::Commit,
    error::{BranchError, RelicError},
    oid::ObjectID,
    state::State,
};

/*
checkout has two functionalities
1. change head to another branch
    check if branch exists
        if yes
            set
        else
            let user know, and suggest to create new branch (-n)
2. create new branch (and change head)
    check if a base branch is provided
        if yes
            use that
        else
            just assume its the current head's branch
            if head is not branch (empty/detached)
                let user know to retry with specified branch name
            else
                create and use as parent commit

*/
pub fn checkout(state: Option<&mut State>, args: &ArgMatches) {
    let Some(state) = state else { return };

    let object_name = args.get_one::<String>("OBJECT").unwrap().clone();
    // determine whether its an object
    if let Some(c) = ObjectID::from_string(&object_name)
        .and_then(|o| o.construct_strict::<Commit>(&state.get_sanctum_path()))
    {
        println!(
            "Detaching head and checking out commit:\n{}.",
            c.get_nickname()
        );

        if let Some(e) = Branch::set_head_detached(c, state) {
            println!("Can't checkout commit: {e:?}");
        }

        return;
    }

    let create_new = args.get_count("new") != 0;
    let base_branch_name = args.get_one::<String>("base");

    let base_branch = match base_branch_name {
        Some(n) => {
            let existing_sources = Branch::construct_from_name_all(state, n);
            if existing_sources.is_empty() {
                println!("Base branch '{n}' could not be found.");
                return;
            }
            Some(existing_sources[0].0.clone())
        }
        None => match Branch::get_head(state, &BranchSource::Local) {
            Ok(HeadType::Branch(b)) => Some(b),
            _ => {
                if create_new {
                    println!("Unable to fetch default base branch. Please specify a branch to base {object_name} off from with the -b flag.");
                    return;
                }
                None
            }
        },
    };
    match Branch::set_head_branch(object_name.clone(), state) {
        Some(err) => match err {
            RelicError::BranchError(BranchError::BranchDoesntExist) => {
                println!("Branch/commit '{object_name}' doesn't exist.");
                if create_new {
                    let base_branch = base_branch.unwrap(); // base_branch is guaranteed to be Some()
                    println!(
                        "Creating new branch ({object_name}) based off your current branch ({}).",
                        base_branch.name
                    );

                    match Branch::instantiate(
                        object_name.clone(),
                        base_branch.get_commit(&state.get_sanctum_path()).ok(),
                        state,
                        &BranchSource::Local,
                    ) {
                        Ok(_) => match Branch::set_head_branch(object_name.clone(), state) {
                            Some(e) => println!("Can't update branch: {e:?}"),
                            None => println!("Successfully changed branch to '{object_name}'."),
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
        None => println!("Successfully changed branch to '{object_name}'."),
    }
}
