use std::path::PathBuf;

use crate::core::{
    data::{commit::Commit, commit_func::CommitState},
    error::{MergeError, RelicError},
    state::State,
    util::get_time,
};

// pub const MERGE_MESSAGE: &str = "Merged"

impl Commit {
    pub fn can_merge(
        base: &Commit,
        feature: &Commit,
        sanctum_path: &PathBuf,
    ) -> (bool, Option<CommitState>) {
        let s = Commit::get_state(base, feature, sanctum_path);
        (
            match s {
                CommitState::Ahead(_) | CommitState::Behind(_) => true,
                CommitState::Divergence(_, _) | CommitState::Tie | CommitState::None => false,
            },
            Some(s),
        )
    }

    pub fn create_merge_commit(
        base: &Commit,
        feature: &Commit,
        state: &State,
    ) -> Result<(Commit, CommitState), RelicError> {
        // creates a commit that merges base and feature together
        // in a branch context, the output commit will
        // be the newest commit on the feature branch

        let (can_merge, commit_state) = Commit::can_merge(base, feature, &state.get_sanctum_path());
        if !can_merge {
            return Err(RelicError::MergeError(MergeError::CantMerge));
        }
        // commit_state will be a some value at this point
        let commit_state = commit_state.unwrap();
        let (parent, surrogate) = match &commit_state {
            CommitState::Ahead(v) => {
                // feature has commits that base doesnt
                assert_eq!(feature.oid, v.last().unwrap().oid);

                // use base as the surrogate
                (feature, base)
            }
            CommitState::Behind(v) => {
                // base has commits that feature doesnt
                assert_eq!(base.oid, v.last().unwrap().oid);

                // use feature as the surrogate
                (base, feature)
            }
            _ => panic!("unmergeable"),
        };

        Ok((
            Commit::new(
                parent.tree,
                Some(parent.oid),
                vec![surrogate.oid],
                get_time(),
                parent.author.clone(),
                format!(
                    "Merge: {} into {}",
                    parent.oid.to_string(),
                    surrogate.oid.to_string()
                ),
                "Merge automatically constructed by relic".to_string(),
                &state.get_sanctum_path(),
            ),
            commit_state,
        ))
    }
}
