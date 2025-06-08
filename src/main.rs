use std::collections::HashMap;
use std::path::PathBuf;

mod paths;

mod content_set;
mod error;
mod state;
mod utils;

mod branch;
mod commit;
mod relic;
mod relic_info;
mod stash;

mod change;
mod content;

use clap::{arg, value_parser, ArgMatches, Command};
use commit::{pending, remove};
use content_set::TrackingSet;
use state::init;
use utils::generate_tree;

use crate::branch::branch;
use crate::commit::{add, cherry, commit, fetch, pull, push, rollback};
use crate::stash::{restore, stash};
use crate::state::State;

// add
// commit {message}
// push
// pull
// fetch
// branch {name}
//      will change to that branch
//      if branch doesnt exist, create
//      ask to create stash (if changes present)
// stash {name|optional}
//      stashes are bound to a branch
//      optional to have a name
// restore
//      select stash to restore
// rollback
//      resets to current head
// cherry {commit hash}

fn main() {
    let mut command_handler = Command::new("relic")
        .about(
            r#"This is the Relic Version Control System.

The best way to learn is to stupidly and
blindly reinvent the wheel.

Relic is a simple hobby project, because
remaking Git sounded fun and interesting.

Most common features like committing,
pushing and pulling, are implemented."#,
        )
        .subcommand_required(true)
        .arg_required_else_help(true);

    type CommandType = fn(&mut State, &ArgMatches);
    let mut commands: HashMap<String, CommandType> = HashMap::new();
    for (f, c) in HashMap::<CommandType, clap::Command>::from_iter::<
        Vec<(CommandType, clap::Command)>,
    >(vec![
        (
            init,
            Command::new("init").about("Initialises a Relic repository in the current directory."),
        ),
        (
            state::clone,
            Command::new("clone").about("Clone a remote Relic repository in the current directory.")
            .arg_required_else_help(true)
            .arg(
                arg!([URL] "URL of the remote Relic repository")
                .required(true)
            )
        ),
        (
            state::detach,
            Command::new("detach").about("Completely removes Relic from the current directory.")
        ),
        (
            add,
            Command::new("add")
                .about("Adds a file(s) to staging")
                .arg_required_else_help(true)
                .arg(
                    arg!([FILE]... "File(s) to add (* for all)")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                ),
        ),
        (
            remove,
            Command::new("remove")
                .about("Removes a file(s) to staging")
                .arg_required_else_help(true)
                .arg(
                    arg!([FILE]... "File(s) to remove (* for all)")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                ),
        ),
        (
            commit,
            Command::new("commit")
                .about("Commit current changes.")
                .arg_required_else_help(true)
                .arg(arg!(-m --message <MESSAGE> "Commit message").required(true))
                .arg(arg!(-d --description <DESCRIPTION> "Commit description")),
        ),
        (
            push,
            Command::new("push").about("Pushes local changes to remote."),
        ),
        (
            pull,
            Command::new("pull").about("Pull changes from remote to local."),
        ),
        (
            fetch,
            Command::new("fetch").about("Check remote for new changes."),
        ),
        (branch, Command::new("branch").about("")),
        (
            stash,
            Command::new("stash")
                // pseudo-commits basically
                // clear stash after a commit
                // stash create
                // stash view
                // stash restore
                // stash delete
                .about(""),
        ),
        (
            restore,
            Command::new("restore"), // unimplemented
        ),
        (
            rollback,
            Command::new("rollback").about("Discard all current changes. Rolls back to most recent commit (or pending commit)."),
        ),
        (
            cherry,
            Command::new("cherry").about("Go to specific commit."),
        ),
        (
            |s, _| {
                println!("{}", generate_tree(&s.current));
            },
            Command::new("tree").about("Generate content tree of current directory."),
        ),
        (
            |s, _| {
                println!(
                    "{}",
                    s.get_changes()
                        .filter_changes(&s.track_set.initialise(&mut s.current))
                        .serialise_changes()
                );
            },
            Command::new("staging").about("View all staging changes."),
        ),
        (
            pending,
            Command::new("pending").about("View all pending commits.")
                .arg(arg!([COMMIT]... "Commit number."))
        ),
        (
            |_, _| {
                println!(".................................................................................................\n.................................................................................................\n.................................................................................................\n.....-----:......:-:.......:--........:-:.......:------::......:----:....:-:....:---::...:::.....\n..:-+#%%%%+=:...:+#=:......=#+:......-*#+:......=%%%%%%#*=:..:=*%%%%*-:.-+#=..-+#%%%#+-.:=#*-....\n.:-%%#+=+#%%+:..-*%+:.....:=%*:.....:+%@#-.....:+%%++++#%%*-.=#%#+=#@#-.-#%=::+%%+=+%%+::+%#-....\n.-#%=:...:=%%=..-*%+:.....:=%*:.....=#%#%*:....:+%*:...:=%%=-+#+:..-%%=.-*%=:-*#=..:=%#-:+%#-....\n:+@*:.....:+%#-.-#%+-.....:+%*:....:+%*=#%-....:+%*-....-#%+:::....-%%=:-#%=:.::...:=%#-:+%#-....\n-#@=:.....:=%%-.-#%+-------+%*:....-##=:+%+:...:+%*:...:=%%=:.....-+%*-.-*%=:.....:=#%+:.=%#-....\n-#@=......:=%%=.-#@%%######%@*:...:+%+:.-##-:..:+%#====+#%*-.....-*%*-..-*%=.....:=#%+-.:=%#-....\n-#@=:......=%%=.-#@#+======*@*:..:=##=::-+@*-..:+@%##%%@#=:.....-#%*-:..-*%=....:+%#=:..:=%#-....\n-*@=:.....:=%%-.:#@+-.....:+%*:..:+@%*++*#@%=..:+%#=--=@#=:....:=%#-....-*#=....-#%+:....=%*-....\n:+@*-.....-*%#:.:#@+:.....:=%*:..=#%######%@*-.:+%*:..:*%*-:...:-+=:....:==-....:++-.....-==:....\n.-#%+-:.:-+%%=..:#%+-.....:=%*:.-*%*::::..=#%=::+%*-...-#%+-.....::......:::.....::.......::.....\n.:-%%%*++#%%=:..-*%+:......+%+-:=%#=:.....-+@*-:+%*-....=%%=:..:=#+-...:-+*=:...:*#=:....=*+-....\n...:=*#%%%@#=...:+*=:......=#=::+#+-.:.::.:-**-:=*=:.::::=#*-:..=*+-....:+*=....:**=.....=*+-....\n....:::::-+%%+:..::........::-::---:::::::::-=-:::::--===+++==-::::......::......::.......::.....\n..........:-=-.............:----==--:::::----===-:-=++*####++++=-:::----:::::::..................\n..:........................:---===-----------==+=--=***+==+***++=--=+++++=--===-.................\n.....::::::................:-====-------------===-:-------=+*****+++===++*+***+=:................\n...:-==++++=--..::---:.....:-=======--======-:-=-......:..::-==++==-:::---====-::................\n...:=+**#*+++=--==+++=-....-=--============-::-=-.......::::::-::::::::::.::::......:::..........\n...::--==+++++++++***+-::.:------====+++===-----:......-===++====---====-:........:-===---::::...\n......::=+**######**+-::...:--========*#+=---::...:::::=+++*####****####*=-:::.:.:=+#***+++===-:.\n......::-==+++++==-:::...:...:--===---=*+-::.:......::-=********+++++****+++====---=++*+****+++==\n..........::::::::::::....:..:.::-==---+*=::...::.:::::-=+*####+=---+###****#**++++=-----=+***+++\n............::--======-------===-------+#+-:::::::::.::::-----=-:::--++*#######*+++++=-:::-=+*###\n..........:.-=+++******++++=+***+=-----=*#-------:::::::-----------=+++********++****+=:::::--=++\n....:.:...::-+***##########*+###+=-----=+#+=====----------=======++*################*+--::.:.::::\n..::::::..::-+*****#+=======-==---=======++====----=--=======+*###****##*+++++++++===-------:::..\n:-=====--:::-=+**###+---------------==++++====---------=======+*#####*++=-::-::::::::-=++++=+==--\n-++*#*++==--==++====---:::--::::::--==========--------======---=++##++=--:::::::::::-=+*****##**+\n:-=+++++*+=+++**=--::::::::--------======--------------=====--=+****+++==--:::::::--=+**+**######\n.::-=++++++**#**=--::::::----========------------------------=+####*++++++++=-::-=++***###******+\n.::-=+**+*##*+=--:::::::--:--========-----=======-----:--------==+**+*+++***+=--+#*******########\n.:::-=======--:::::::::::-----=========++#*******+=--:::::::::::----==++****+++=+#*****++===+++++\n.:.:::::::::::::::::::::::----======++############*+--::::::::::::::---=********+******+=--::::::\n....:.::::---=====---::::------====+*#############**+=--::::::::::::::--=*#************=--::::::.\n.:::---==++++++**++++=--::--------=+#######*+++##******=======--::::::::-=+*#########*+=======--:\n:--=+++**************++---::::-----+*######*===+*#***#*+++****++-:::::::::--===++++==*******+++++\n=++*****######********+--:::::::---=*#####*+=--=+**###+*#******+-::::::::.:::::::::-=+#########**\n+******###************+--:::::::::--=+***+=-------=++==*#####*+=-:::..:..::::..:::::--=+*#######*\n******##*==++********+=--:::::::::::---==--::::::------===++++=-:::::...:..:.::..:::-==*+********\n#######*==+*********+--:::::::.::::::::..:::::--=+**+=---::::::::...::........:::--=+###*********\n##**+==--=+*****###+=--::::...::.:..:.::..:::-++*****#*=--:::::....:...:..:.:::-=++**############\n=--------=+########*+=--::::::..:.:.:..:::::-=**#**####**+=-::::::::--:..::.::-=*#*******#*++++++\n::::::::--+#####******+=-:::.:.............::-==+#####*****+-::::-==++=-::.:::-=*####***+=-------\n.:.:.::::--+**********#*+-:::............::.:::--==++**#****=--=++*****+-:::.::--=++*#*==--::::::\n.......:::-=+****#*####*+-::::..............::::::--=+******=--+*#*#****=-:::.::::--==---::::....\n......::::--=+*######*+==-::.........::.:.....:::::-=+******+===****#***=-:::::.::::::::.........\n......:.::::--=***+==--:::...............::.....:::-=+**##********##***+=-:::....................\n....:::.:::::--=---:::::.......................:.::-=+#**####****##***+=--::.:...................\n................::.::..........................:.::--=*##########***++=-:::......................");
            },
            Command::new("qhar").about("??")
        ),
        (
            |s, _| {
                println!("{:?}", s.info);
            },
            Command::new("test").about("this is here for debug purposes")
        )
    ]) {
        commands.insert(c.get_name().to_string(), f);
        command_handler = command_handler.subcommand(c);
    }

    let s = State::create(PathBuf::from("."));

    let c = command_handler.get_matches();
    let (command_name, sub_matches) = c.subcommand().unwrap();

    // TODO : shorten and undry this
    if let Ok(mut s) = s {
        match command_name {
            "clone" | "init" => {
                // "init" => {
                // let this run only for
                // clone, init
                println!("Unable to '{command_name}' an already existing Relic repository.");
                return;
            }
            _ => match commands.get(command_name) {
                Some(command) => {
                    command(&mut s, sub_matches);
                }
                None => {
                    unimplemented!("Relic Error, command not defined.");
                }
            },
        }
    } else {
        match command_name {
            "clone" | "init" => {
                // let this run only for
                // clone, init
                match commands.get(command_name) {
                    Some(command) => {
                        command(&mut State::empty(), sub_matches);
                    }
                    None => {
                        unimplemented!("Relic Error, command not defined.");
                    }
                }
            }
            _ => {
                println!("No valid Relic repository found in current directory. Consider executing 'relic init' or 'relic clone'.");
                return;
            }
        }
    }
}
