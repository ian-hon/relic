use std::path::Path;

mod core;

mod cli;
mod commands;

/*
get ref system working


*/

fn main() {
    let cli = cli::build();
    cli::handle(cli, Path::new("."));
}
