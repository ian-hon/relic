use std::path::Path;

mod core;

mod cli;
mod commands;

/*
get ref system working


*/

fn main() {
    let command_handler = cli::build();
    let args = command_handler.handler.clone().get_matches();
    let relic_path = Path::new("./.relic");

    cli::handle(
        command_handler,
        args,
        if relic_path.exists() {
            Some(relic_path)
        } else {
            None
        },
    );
}
