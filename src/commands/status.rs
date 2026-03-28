use clap::ArgMatches;

use crate::core::{
    branch::branch::Branch,
    data::{commit::Commit, commit_func::CommitState},
    error::RELIC_ERROR_CORRUPTED,
    state::State,
};

pub fn status(state: Option<&mut State>, args: &ArgMatches) {
    let Some(state) = state else { return };

    let pair = match (
        args.get_one::<String>("BASE"),
        args.get_one::<String>("FEATURE"),
    ) {
        (Some(base_name), Some(feature_name)) => {
            let base_branches = Branch::construct_from_name_all(state, base_name);
            let Some(base_branch) = base_branches.first() else {
                println!("No branch exists with name \"{base_name}\"");
                return;
            };
            let Ok(base_commit) = base_branch.0.get_commit(&state.get_sanctum_path()) else {
                println!("Unable to fetch commit from {base_name}");
                return;
            };

            let feature_branches = Branch::construct_from_name_all(state, feature_name);
            let Some(feature_branch) = feature_branches.first() else {
                println!("No branch exists with name \"{feature_name}\"");
                return;
            };
            let Ok(feature_commit) = feature_branch.0.get_commit(&state.get_sanctum_path()) else {
                println!("Unable to fetch commit from {feature_name}");
                return;
            };

            (
                (base_commit, base_name.as_str()),
                (feature_commit, feature_name.as_str()),
            )
        }
        (Some(base_name), None) => {
            let base_commits = Branch::construct_from_name_all(state, base_name)
                .iter()
                .filter_map(|b| b.0.get_commit(&state.get_sanctum_path()).ok())
                .collect::<Vec<Commit>>();
            if base_commits.len() != 2 {
                println!("Can't fetch upstream/local branch/commits. {RELIC_ERROR_CORRUPTED}");
                return;
            }

            (
                (base_commits[0].clone(), "local"),
                (base_commits[1].clone(), "upstream"),
            )
        }
        (None, Some(_)) => panic!(),
        (None, None) => {
            let (Ok(local), Ok(upstream)) = (
                state.fetch_local_head_commit(),
                state.fetch_upstream_head_commit(),
            ) else {
                println!("Cant seem to get either upstream or head. {RELIC_ERROR_CORRUPTED}");
                return;
            };
            let (Some(local), Some(upstream)) = (local, upstream) else {
                println!("No pending commits.");
                return;
            };
            ((local, "Local"), (upstream, "Upstream"))
        }
    };

    match Commit::get_state(&pair.1 .0, &pair.0 .0, &state.get_sanctum_path()) {
        CommitState::Ahead(v) => {
            println!("{} is ahead by {} commits.", pair.0 .1, v.len());
            for c in v {
                println!("{}", c.get_nickname(false));
            }
        }
        CommitState::Behind(v) => {
            println!("{} is behind by {} commits.", pair.0 .1, v.len());
            for c in v {
                println!("{}", c.get_nickname(false));
            }
        }
        CommitState::Tie => {
            println!("{} is up to date with {}.", pair.0 .1, pair.1 .1);
        }
        CommitState::Divergence(ancestor, _) => {
            println!(
                "Divergence between {} and {}. Last common ancestor:\n{}",
                pair.0 .1,
                pair.1 .1,
                ancestor.get_nickname(false)
            );
        }
        CommitState::None => {
            println!(
                "{} and {} are not related. {RELIC_ERROR_CORRUPTED}",
                pair.0 .1, pair.1 .1
            );
        }
    }
}
