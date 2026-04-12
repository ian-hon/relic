use std::path::{Path, PathBuf};

use crate::{commands as command_module, core::state::State};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "Relic")]
#[command(about = r#"This is the Relic Version Control System.

Relic is a CAS + Merkle DAG (Directed
Acyclic Graph) using SHA256. Objects are
stored inside ./.relic/sanctum/.

I wanted to truly understand how Git
works, so I made Relic. Everyone knows
the best way to learn is to stupidly and
naively reinvent the wheel."#)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialises a Relic repository in the current directory.
    Init(command_module::init::InitArgs),

    /// Clone a remote Relic repository in the current directory.
    Clone(command_module::clone::CloneArgs),

    /// Completely removes Relic from the current directory.
    Detach(command_module::detach::DetachArgs),

    /// View all staging changes.
    Staging(command_module::staging::StagingArgs),

    /// Adds paths to be tracked
    Track(command_module::track::TrackArgs),

    /// Removes paths from being tracked
    Untrack(command_module::untrack::UntrackArgs),

    /// View all pending commits.
    Pending(command_module::pending::PendingArgs),

    /// Commit current changes.
    Commit(command_module::commit::CommitArgs),

    /// Pushes pending commits to remote.
    Push(command_module::push::PushArgs),

    /// Pull pending commits from remote to local.
    Pull(command_module::pull::PullArgs),

    /// Switches to specified branch. Use -n to create new branch if it doesn't exist.
    Checkout(command_module::checkout::CheckoutArgs),

    /// Merges the selected branch into the current branch.
    Merge(command_module::merge::MergeArgs),

    /// Generate content tree of current directory.
    Tree(command_module::tree::TreeArgs),

    /// View status between branches.
    Status(command_module::status::StatusArgs),

    /// ??
    Qhar(command_module::qhar::QharArgs),

    /// This is here for debug purposes
    Test(command_module::test::TestArgs),
}

pub fn build() -> Cli {
    Cli::parse()
}

pub fn handle(cli: Cli, path: &Path) {
    match cli.command {
        Commands::Init(args) => {
            command_module::init(&path.to_path_buf(), args);
        }
        Commands::Clone(args) => {
            command_module::clone(args);
        }
        _ => {
            let Some(mut state) = State::construct(path.into()) else {
                println!("No valid Relic repository found in current directory. Consider executing 'relic init' or 'relic clone'.");
                return;
            };

            match cli.command {
                Commands::Init(_) | Commands::Clone(_) => unreachable!(),
                Commands::Detach(args) => {
                    command_module::detach(&mut state, args);
                }
                Commands::Staging(args) => {
                    command_module::staging(&mut state, args);
                }
                Commands::Track(args) => {
                    command_module::track(&mut state, args);
                }
                Commands::Untrack(args) => {
                    command_module::untrack(&mut state, args);
                }
                Commands::Pending(args) => {
                    command_module::pending(&mut state, args);
                }
                Commands::Commit(args) => {
                    command_module::commit(&mut state, args);
                }
                Commands::Push(args) => {
                    command_module::push(&mut state, args);
                }
                Commands::Pull(args) => {
                    command_module::pull(&mut state, args);
                }
                Commands::Checkout(args) => {
                    command_module::checkout(&mut state, args);
                }
                Commands::Merge(args) => {
                    command_module::merge(&mut state, args);
                }
                Commands::Tree(args) => {
                    command_module::tree(&mut state, args);
                }
                Commands::Status(args) => {
                    command_module::status(&mut state, args);
                }
                Commands::Qhar(args) => {
                    command_module::qhar(&mut state, args);
                }
                Commands::Test(args) => {
                    command_module::test(&mut state, args);
                }
            }
        }
    }
}
