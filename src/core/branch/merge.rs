use crate::core::{
    branch::branch::{Branch, BranchSource},
    data::{commit::Commit, commit_func::CommitState},
    error::{BranchError, RelicError},
    state::State,
};

impl Branch {
    pub fn merge(
        base: (&Branch, &BranchSource),
        feature: (&Branch, &BranchSource),
        state: &State,
    ) -> Result<Commit, RelicError> {
        let Ok(base_commit) = base.0.get_commit(&state.get_sanctum_path()) else {
            return Err(RelicError::ConfigurationIncorrect);
        };

        let Ok(feature_commit) = feature.0.get_commit(&state.get_sanctum_path()) else {
            return Err(RelicError::ConfigurationIncorrect);
        };

        match Commit::create_merge_commit(&base_commit, &feature_commit, state) {
            Ok((merge_commit, commit_state)) => {
                // update branches
                // Branch::update_branch(name, new_commit, state, source)

                let (updated_branch, updated_branch_source) = match commit_state {
                    // Ahead = feature has changes base doesnt
                    // new merge commit is on base branch
                    CommitState::Ahead(_) => base,
                    _ => panic!(),
                };

                if let Err(e) = Branch::update_branch(
                    &updated_branch.name,
                    &merge_commit,
                    state,
                    updated_branch_source,
                ) {
                    return Err(e);
                }

                Ok(merge_commit)
            }
            Err(e) => Err(e),
        }
    }

    pub fn merge_from_name(
        base: (&str, &BranchSource),
        feature: (&str, &BranchSource),
        state: &State,
    ) -> Result<Commit, RelicError> {
        if let (Some(base_branch), Some(feature_branch)) = (
            Branch::construct_from_name(base.0, state, base.1),
            Branch::construct_from_name(feature.0, state, feature.1),
        ) {
            return Branch::merge((&base_branch, base.1), (&feature_branch, feature.1), state);
        }
        Err(RelicError::BranchError(BranchError::BranchDoesntExist))
    }
}
