use crate::core::{
    data::{commit::Commit, commit_func::CommitState},
    error::{MergeError, RelicError},
    object::ObjectLike,
    state::State,
    util::get_time,
};

impl Commit {
    pub fn create_merge_commit(
        base: &Commit,
        feature: &Commit,
        state: &State,
    ) -> Result<(Commit, CommitState), RelicError> {
        // creates a commit that merges base and feature together
        // in a branch context, the output commit will
        // be the newest commit on the feature branch

        let commit_state = Commit::get_state(base, feature, &state.get_sanctum_path());
        match &commit_state {
            CommitState::Ahead(v) => {
                // feature has commits that base doesnt
                assert_eq!(feature.oid, v.last().unwrap().oid);

                let c = Commit::new(
                    feature.tree,
                    Some(base.oid),
                    vec![feature.oid],
                    get_time(),
                    base.author.clone(),
                    format!(
                        "Merge: {} into {}",
                        feature.oid.to_string(),
                        base.oid.to_string()
                    ),
                    "Merge automatically constructed by relic".to_string(),
                    vec![],
                );

                c.write(&state.get_sanctum_path());

                // add a merge commit on base
                // use feature's tree
                Ok((c, commit_state))
            }
            CommitState::Behind(_) => {
                return Err(RelicError::MergeError(MergeError::AlreadyContainsChanges))
            }
            CommitState::Tie => return Err(RelicError::MergeError(MergeError::AlreadyEqual)),
            _ => return Err(RelicError::MergeError(MergeError::UnresolvedConflicts)),
        }
    }
}
