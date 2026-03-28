use std::{collections::HashSet, path::Path};

use crate::core::{data::commit::Commit, object::ObjectLike};

impl Commit {
    pub fn get_state(upstream: &Commit, local: &Commit, sanctum_path: &Path) -> CommitState {
        // only care about HEAD
        // if l.head is inside u_set => Behind
        // if u.head is inside l_set => Ahead
        // if neither => None OR Divergence
        //      find the last common commit between upstream and local
        //           if none exists => None

        // i dont think we need to care about the surrogate parents
        // (emphasis on think)

        if upstream.get_oid() == local.get_oid() {
            return CommitState::Tie;
        }

        let mut u_all = upstream.get_all_parents(sanctum_path);
        let mut l_all = local.get_all_parents(sanctum_path);
        // EXPENSIVE!
        u_all.reverse();
        l_all.reverse();

        let u_set: HashSet<[u8; 32]> = HashSet::from_iter(u_all.iter().map(|x| x.get_oid().0));
        let l_set: HashSet<[u8; 32]> = HashSet::from_iter(l_all.iter().map(|x| x.get_oid().0));

        if l_set.contains(&upstream.get_oid().0) {
            if let Some((_, i)) = Commit::get_last_common(&u_all, &l_all) {
                return CommitState::Ahead(l_all[(i + 1)..].to_vec());
            }
            panic!("no common found: Ahead");
        }

        if u_set.contains(&local.get_oid().0) {
            if let Some((_, i)) = Commit::get_last_common(&u_all, &l_all) {
                return CommitState::Behind(u_all[(i + 1)..].to_vec());
            }
            panic!("no common found: Behind");
        }

        if let Some((c, index)) = Commit::get_last_common(&u_all, &l_all) {
            // technically speaking there shouldnt be oob error here
            // if index + 1 doesnt exist, that means its either behind or ahead; not divergent
            return CommitState::Divergence(
                c.clone(),
                (u_all[(index + 1)..].to_vec(), l_all[(index + 1)..].to_vec()),
            );
        }
        CommitState::None
    }

    fn get_last_common(a: &Vec<Commit>, b: &Vec<Commit>) -> Option<(Commit, usize)> {
        // for a and b, oldest commit to newest commit
        // can use binary search here to speed things up
        let mut previous = None;
        for index in 0..(a.len().min(b.len())) {
            if a[index].get_oid() != b[index].get_oid() {
                return previous;
            }
            previous = Some((a[index].clone(), index));
        }
        previous
    }
}

#[derive(Debug)]
pub enum CommitState {
    Ahead(Vec<Commit>), // local has commits that upstream doesnt
    /*
    Upstream: A > B > C
    Local   : A > B > C > D > E

    Value stored in the Vec (in order): [D, E]
    */
    Behind(Vec<Commit>), // upstream has commits that local doesnt
    /*
    Upstream: A > B > C > D > E
    Local   : A > B > C

    Value stored in the Vec (in order): [D, E]
     */
    Tie, // both are equal
    /*
    Upstream: A > B > C
    Local   : A > B > C
     */
    Divergence(Commit, (Vec<Commit>, Vec<Commit>)), // upstream and local have diverging commits
    // Divergence({last common commit})
    /*
    Upstream: A > B > C > D > E
    Local   : A > B > C > F > G

    Two types of divergence:
        Resolved
            There are no divergence in the changes between upstream and local
            Basically, upstream and local did not modify any of the same files

            What to do:
                F.parent = E
                                    U   U   L   L
                Result: A > B > C > D > E > F > G

        Unresolved
            There are divergence in the changes between upstream and local
            Upstream and local modified the same files
            Dont know whether to use upstream's or local's changes

            What to do:
                F.parent = E

                Create new commit (H) to resolve this
                User chooses which changes (in conflicted file) to apply

                                    U   U   L   L  Fix
                Result: A > B > C > D > E > F > G > H
    */
    None, // cant detect correlation between these commits
          /*
          Upstream: A > B > C
          Local   : X > Y > Z
          */
}
