extern crate notify;

use clap::{arg, Arg, Command as ClapCommand};

pub fn parse_args() -> clap::ArgMatches {
    let matches = ClapCommand::new("tcr")
        .trailing_var_arg(true)
        .version("1.0")
        .author("Igor Brejc")
        .about("Runs 'test && commit || revert' workflow.")
        .arg(
            Arg::new("TEST STEP")
                .long("test")
                .short('t')
                .required(false)
                .takes_value(true)
                .help(
                    "The command to run as a test step. If not specified, \
                    only a warning will be printed during the test step.",
                )
                .display_order(1),
        )
        .arg(
            arg!(-p --path <PATH>)
                .required(false)
                .default_value(".")
                .help("The path to watch for file changes.")
                .display_order(2),
        )
        .arg(
            Arg::new("FILE PATTERN")
                .long("file-pattern")
                .short('f')
                .required(false)
                .default_value(".*.rs")
                .requires("TEST STEP")
                .help(
                    "The regex file pattern which changed/created/deleted \
                    files must match to trigger the test step.",
                )
                .display_order(3),
        )
        .arg(
            arg!(-d --delay <DELAY>)
                .required(false)
                .default_value("1000")
                .help(
                    "The delay (in milliseconds) between the first detected k\
                    file change and running the test step.",
                )
                .validator(|s| s.parse::<u64>()),
        )
        .arg(
            Arg::new("TEST COMMAND ARGS")
                .multiple_values(true)
                .required(false),
        )
        .get_matches();
    matches
}
