use clap::ArgMatches;

use crate::core::state::State;

pub fn pending(state: &mut State, args: &ArgMatches) {
    let pending = state.pending_get();

    if let Some(commit_number) = args
        .get_one::<String>("COMMIT")
        .map_or(None, |x| x.parse::<i32>().map_or(None, |x| Some(x)))
    {
        // display selected
        if (commit_number < 0) || (commit_number >= pending.len() as i32) {
            println!(
                "Invalid selection. Please select commit numbers in the range of (0-{})",
                pending.len() - 1
            );
            return;
        }

        println!("{}", pending[commit_number as usize].serialise());
    } else {
        // display all
        for (index, c) in pending.iter().enumerate() {
            println!("{index}. {}", c.header());
        }
    }
}
