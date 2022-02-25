extern crate notify;

use clap::{arg, Arg, Command as ClapCommand};
use notify::DebouncedEvent::{Create, Remove, Rename, Write};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use regex::RegexSet;
use std::io;
use std::io::Write as IoWrite;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;

fn main() {
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
        .get_matches();

    let test_step = matches.value_of("test");
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

    loop {
        match rx.recv() {
            Ok(event) => {
                handle_watch_event(&event, &matching_files, &test_step)
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

// todo now: collect all of the updates and then run TCR
fn handle_watch_event(
    event: &DebouncedEvent,
    matching_files: &RegexSet,
    test_step: &Option<&str>,
) {
    let event_data: Option<(String, String)> = match event {
        Create(path) => extract_event_data("create", &path, matching_files),
        Remove(path) => extract_event_data("remove", &path, matching_files),
        Rename(from_path, _) => {
            extract_event_data("rename", &from_path, matching_files)
        }
        Write(path) => extract_event_data("write", &path, matching_files),
        _ => None,
    };

    if event_data.is_some() {
        println!("{:?}", event_data.unwrap());
        // todo now: how to support the '--' argument separator?
        println!("TEST");

        match test_step {
            Some(test_command) => run_test(test_command),
            None => {
                println!("The test step has not been specified, doing nothing.")
            }
        }
    }
}

fn run_test(test_command: &str) {
    let output = Command::new(test_command)
        .output()
        .expect("failed to execute the test step");
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
) -> Option<(String, String)> {
    if is_path_matched(&path, matching_files) {
        Some((event_desc.to_string(), path_to_str(&path)))
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
