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

*/

pub mod clone;
pub mod commit;
pub mod detach;
pub mod init;
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

pub use clone::clone;
pub use commit::commit;
pub use detach::detach;
pub use init::init;
pub use pending::pending;
pub use pull::pull;
pub use push::push;
pub use qhar::qhar;
pub use staging::staging;
pub use status::status;
pub use test::test;
pub use track::track;
pub use tree::tree;
pub use untrack::untrack;
