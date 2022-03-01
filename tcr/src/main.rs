extern crate notify;

use clap::{arg, Arg, Command as ClapCommand, Values};
use notify::DebouncedEvent::{Create, Remove, Rename, Write};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use regex::RegexSet;
use std::io::Write as IoWrite;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::{fmt, io};

struct SourceCodeUpdateEvent {
    path: String,
    event_type: String,
}

impl fmt::Display for SourceCodeUpdateEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.event_type, self.path)
    }
}

fn main() {
    let matches = parse_args();

    let test_step = matches.value_of("TEST STEP");
    println!("Test step: {}", test_step.unwrap_or("<none>"));

    let test_cmd_args = matches.values_of("TEST COMMAND ARGS");

    let xxx = match test_cmd_args {
        Some(ref args) => args.len(),
        None => 0,
    };
    println!("Test command args len: {}", xxx);

    let file_pattern = matches.value_of("FILE PATTERN").unwrap();

    let matching_files = RegexSet::new(&[file_pattern]).unwrap();

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let delay = matches.value_of("delay").unwrap().parse::<u64>().unwrap();
    let watch_period = Duration::from_millis(delay);

    let mut watcher = watcher(tx, watch_period).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    let path_to_watch = matches.value_of("path").unwrap();

    watcher
        .watch(path_to_watch, RecursiveMode::Recursive)
        .unwrap();

    let mut collected_events: Vec<SourceCodeUpdateEvent> = Vec::new();

    loop {
        match rx.recv() {
            Ok(event) => {
                // todo now: Instead of directly handling each individual event,
                // they should be stored in a collection and handled in a single
                // go, as soon as the 'delay' period from the first event time
                // has passed.
                handle_watch_event(
                    &event,
                    &matching_files,
                    &test_step,
                    &test_cmd_args,
                    &mut collected_events,
                );
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn parse_args() -> clap::ArgMatches {
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

// todo now: collect all of the updates and then run TCR
fn handle_watch_event(
    event: &DebouncedEvent,
    matching_files: &RegexSet,
    test_step: &Option<&str>,
    test_cmd_args: &Option<Values>,
    collected_events: &mut Vec<SourceCodeUpdateEvent>,
) {
    let event_data: Option<SourceCodeUpdateEvent> = match event {
        Create(path) => extract_event_data("create", &path, matching_files),
        Remove(path) => extract_event_data("remove", &path, matching_files),
        Rename(from_path, _) => {
            extract_event_data("rename", &from_path, matching_files)
        }
        Write(path) => extract_event_data("write", &path, matching_files),
        _ => None,
    };

    match event_data {
        Some(event_data) => {
            println!("{}", event_data);
            collected_events.push(event_data);

            // println!("TEST");

            // match test_step {
            //     Some(test_command) => run_test(test_command, test_cmd_args),
            //     _ => {
            //         println!("The test step has not been specified, doing nothing.")
            //     }
            // }
        }
        None => (),
    }
}

fn run_test(test_command: &str, test_cmd_args: &Option<Values>) {
    let mut command = Command::new(test_command);

    let args_str: Option<Vec<String>> = match test_cmd_args {
        Some(args) => {
            let args_str = args
                .clone()
                .into_iter()
                .map(|x| x.to_owned())
                .collect::<Vec<String>>();
            Some(args_str)
        }
        None => None,
    };

    if args_str.is_some() {
        command.args(args_str.unwrap());
    }

    // todo now: print out the running command

    let output = command.output().expect("failed to execute the test step");
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
    // todo now: check the test exit code.
    // In case of success, run commit.
    // In case of failure, run reset.
}

fn extract_event_data(
    event_desc: &str,
    path: &Path,
    matching_files: &RegexSet,
) -> Option<SourceCodeUpdateEvent> {
    if is_path_matched(&path, matching_files) {
        Some(SourceCodeUpdateEvent {
            path: path_to_str(&path),
            event_type: event_desc.to_string(),
        })
    } else {
        None
    }
}

fn is_path_matched(path: &Path, matching_files: &RegexSet) -> bool {
    return matching_files.is_match(&path_to_str(&path));
}

fn path_to_str(path: &Path) -> String {
    let os_path_str = path.as_os_str();
    os_path_str.to_str().unwrap().to_string()
}
