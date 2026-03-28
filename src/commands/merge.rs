use clap::ArgMatches;

use crate::core::{
    branch::branch::{Branch, BranchSource, HeadType},
    error::{MergeError, RelicError},
    state::State,
};

pub fn merge(state: Option<&mut State>, args: &ArgMatches) {
    let Some(state) = state else { return };

    let feature_name = args.get_one::<String>("BRANCH").unwrap();
    let feature_branches = Branch::construct_from_name_all(state, feature_name);
    let Some(feature) = feature_branches.first() else {
        println!("No branch exists with name \"{feature_name}\"");
        return;
    };
    let base = match Branch::get_head(state, &BranchSource::Local) {
        Ok(h) => match h {
            HeadType::Branch(b) => b,
            HeadType::Detached(_) => {
                println!("HEAD is currently detached. No branch is active. Use `relic checkout` to set an active branch.");
                return;
            }
            HeadType::Empty => {
                println!("HEAD is unset. Use `relic checkout` to set an active branch.");
                return;
            }
        },
        Err(e) => {
            println!("Unable to fetch HEAD: {:?}", e);
            return;
        }
    };

    match Branch::merge(
        (&base, &BranchSource::Local),
        (&feature.0, &feature.1),
        state,
    ) {
        Ok(c) => {
            println!(
                "Merged {} into {} successfully. Both branches are now equal. Commit ({})",
                feature_name,
                base.name,
                c.oid.to_string()
            )
        }
        Err(e) => {
            println!(
                "{}",
                match e {
                    RelicError::MergeError(MergeError::UnresolvedConflicts) =>
                        "Unable to merge with unresolved conflicts.".to_string(),
                    RelicError::MergeError(MergeError::AlreadyContainsChanges) => format!(
                        "{} already has all of {}'s changes",
                        base.name, feature.0.name
                    ),
                    RelicError::MergeError(MergeError::AlreadyEqual) => format!(
                        "{} is already up to date with {}",
                        base.name, feature.0.name
                    ),
                    _ => format!("Unable to merge: {e:?}"),
                }
            );
            return;
        }
    }
}
