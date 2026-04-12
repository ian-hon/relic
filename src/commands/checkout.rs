use crate::core::{
    branch::branch::{Branch, BranchSource, HeadType},
    data::{commit::Commit, commit_func::CommitState, tree::Tree},
    error::{BranchError, RelicError, RELIC_ERROR_CORRUPTED},
    oid::ObjectID,
    state::State,
    write,
};
use clap::Args;

#[derive(Args)]
pub struct CheckoutArgs {
    /// Branch/commit to checkout
    #[arg(conflicts_with_all = ["all", "local", "upstream"])]
    pub object: Option<String>,

    /// Create new if no exist
    #[arg(short, long, action = clap::ArgAction::Count, conflicts_with_all = ["all", "local", "upstream"])]
    pub new: u8,

    /// Branch to base from
    #[arg(short, long, conflicts_with_all = ["all", "local", "upstream"])]
    pub base: Option<String>,

    /// Show all available branches
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub all: u8,

    /// List all local branches
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub local: u8,

    /// List all upstream branches
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub upstream: u8,
}

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
pub fn checkout(state: &mut State, args: CheckoutArgs) {
    let (list_all, list_upstream, list_local) =
        (args.all != 0, args.upstream != 0, args.local != 0);

    if list_all || list_upstream || list_local {
        let Ok(branches) = Branch::get_all_branches(state) else {
            println!("Unable to fetch all branches. {RELIC_ERROR_CORRUPTED}");
            return;
        };

        let active = Branch::get_head(state, &BranchSource::Local)
            .ok()
            .map_or(None, |h| match h {
                HeadType::Branch(b) => Some(b.name),
                _ => None,
            });

        for (b, m) in branches.iter() {
            // println!("{b} : {} {m:?}", !m.contains_key(&BranchSource::Local));
            let s = if !list_all {
                if (list_upstream && !m.contains_key(&BranchSource::Upstream))
                    || (list_local && !m.contains_key(&BranchSource::Local))
                {
                    continue;
                }

                if list_upstream {
                    BranchSource::Upstream
                } else {
                    BranchSource::Local
                }
            } else {
                if !m.contains_key(&BranchSource::Local) {
                    BranchSource::Upstream
                } else {
                    BranchSource::Local
                }
            };

            /*
            main (5 commits ahead of remote)
            rewrite (5 commits behind main)
            another
            remote/main
            remote/rewrite
             */

            println!(
                "{}{}",
                if active.as_ref().is_some_and(|a| a.eq(b)) {
                    "(HEAD) "
                } else {
                    "       "
                },
                match s {
                    BranchSource::Local => {
                        // format!("{b}", match commit::Commit::get_state(upstream, local, sanctum_path))
                        let suffix = if let Some(upstream) = m.get(&BranchSource::Upstream) {
                            let local_commit = m
                                .get(&BranchSource::Local)
                                .unwrap()
                                .clone()
                                .get_commit(&state.get_sanctum_path())
                                .unwrap();
                            match Commit::get_state(
                                &upstream
                                    .clone()
                                    .get_commit(&state.get_sanctum_path())
                                    .unwrap(),
                                &local_commit,
                                &state.get_sanctum_path(),
                            ) {
                                CommitState::Ahead(commits) => {
                                    format!("({} commits ahead of upstream)", commits.len())
                                }
                                CommitState::Behind(commits) => {
                                    format!("({} commits behind upstream)", commits.len())
                                }
                                CommitState::Divergence(luca, remainder) => {
                                    format!(
                                        "({} ahead, {} behind; diverged at commit {{{}}})",
                                        remainder.1.len(),
                                        remainder.0.len(),
                                        luca.oid.as_trunc()
                                    )
                                }
                                CommitState::None => "(completely detached (how lmao))".to_string(),
                                CommitState::Tie => "".to_string(),
                            }
                        } else {
                            "".to_string()
                        };

                        format!("{b} {suffix}")
                    }
                    BranchSource::Upstream => format!("remote/{b}"),
                }
            );
        }

        return;
    }

    // determine whether its an object
    let head = Branch::get_head(state, &BranchSource::Local);
    let Some(object_name) = args.object else {
        if let Ok(h) = head {
            println!("Current HEAD: {}", h.as_human_readable());
        } else {
            println!("Can't fetch HEAD. {RELIC_ERROR_CORRUPTED}");
        }
        return;
    };
    if let Some(c) = ObjectID::from_string(&object_name)
        .and_then(|o| o.construct_strict::<Commit>(&state.get_sanctum_path()))
    {
        println!(
            "Detaching head and checking out commit:\n{}.",
            c.get_nickname(false)
        );

        if let Some(e) = Branch::set_head_detached(c, state) {
            println!("Can't checkout commit: {e:?}");
        }

        if let Ok(h) = head {
            if let Ok(Some(c)) = h.get_commit(&state.get_sanctum_path()) {
                write::write_tree(
                    &state.root_path.join("playground"),
                    &state.get_sanctum_path(),
                    &c.tree
                        .construct_strict::<Tree>(&state.get_sanctum_path())
                        .unwrap(),
                );
            }
        }

        return;
    }

    let create_new = args.new != 0;
    let base_branch_name = args.base.as_deref();

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
    match Branch::set_head_branch(object_name.to_string(), state) {
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
                        object_name.to_string(),
                        base_branch.get_commit(&state.get_sanctum_path()).ok(),
                        state,
                        &BranchSource::Local,
                    ) {
                        Ok(_) => match Branch::set_head_branch(object_name.to_string(), state) {
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

    if let Ok(h) = Branch::get_head(state, &BranchSource::Local) {
        if let Ok(Some(c)) = h.get_commit(&state.get_sanctum_path()) {
            write::write_tree(
                &state.root_path.join("playground"),
                &state.get_sanctum_path(),
                &c.tree
                    .construct_strict::<Tree>(&state.get_sanctum_path())
                    .unwrap(),
            );
        }
    }
}
