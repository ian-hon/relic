use clap::Args;

#[derive(Args)]
pub struct CloneArgs {
    /// URL of the remote Relic repository
    pub url: String,
}

pub fn clone(args: CloneArgs) {}
