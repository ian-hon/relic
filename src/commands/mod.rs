/*
staging -> pending -> remote

misc:
init
    create relic repo
detach
    remove relic repo; confirmation needed
clone
    clone repo from URL
status
    show status for staging, pending and remote

staging:
staging
    view diffs
track
    track files
untrack
    untrack files

pending:
pending
    view all pending commits (not on remote yet)
commit
    creates new commit based on changes

remote:
push
    pushes commits to remote
pull
    pulls commits from remote

branch:
checkout
    changes checked out branch to new one, if doesnt exist, creates it

*/

pub mod checkout;
pub mod clone;
pub mod commit;
pub mod detach;
pub mod init;
pub mod merge;
pub mod pending;
pub mod pull;
pub mod push;
pub mod qhar;
pub mod staging;
pub mod status;
pub mod test;
pub mod track;
pub mod tree;
pub mod untrack;

pub use checkout::{checkout, CheckoutArgs};
pub use clone::{clone, CloneArgs};
pub use commit::{commit, CommitArgs};
pub use detach::{detach, DetachArgs};
pub use init::{init, InitArgs};
pub use merge::{merge, MergeArgs};
pub use pending::{pending, PendingArgs};
pub use pull::{pull, PullArgs};
pub use push::{push, PushArgs};
pub use qhar::{qhar, QharArgs};
pub use staging::{staging, StagingArgs};
pub use status::{status, StatusArgs};
pub use test::{test, TestArgs};
pub use track::{track, TrackArgs};
pub use tree::{tree, TreeArgs};
pub use untrack::{untrack, UntrackArgs};
