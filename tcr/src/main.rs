extern crate notify;

mod args;

use clap::ArgMatches;
use notify::DebouncedEvent::{Create, Remove, Rename, Write};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use regex::RegexSet;
use std::io::Write as IoWrite;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fmt, io, thread};

pub use args::parse_args;

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
    let matches = args::parse_args();

    let test_step = get_test_step(&matches);
    let test_cmd_args = get_test_cmd_args(&matches);
    let file_pattern = matches.value_of("FILE PATTERN").unwrap();

    let matching_files = RegexSet::new(&[file_pattern]).unwrap();

    let delay = matches.value_of("delay").unwrap().parse::<u64>().unwrap();
    let watch_period = Duration::from_millis(delay);

    let collected_events: Arc<Mutex<Vec<SourceCodeUpdateEvent>>> =
        Arc::new(Mutex::new(Vec::new()));
    let (tx_collected_events, rx_collected_events) = channel();

    let collected_events_thread = Arc::clone(&collected_events);
    thread::spawn(move || loop {
        {
            let mut collected_events = collected_events_thread.lock().unwrap();

            let mut events_to_process: Vec<SourceCodeUpdateEvent> = Vec::new();
            while collected_events.len() > 0 {
                let event = collected_events.pop().unwrap();
                events_to_process.push(event);
            }
            match tx_collected_events.send(events_to_process) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error sending source code change events: {}", e);
                }
            }
        }

        println!("SLEEPER THREAD");
        thread::sleep(watch_period);
    });

    thread::spawn(move || loop {
        match rx_collected_events.recv() {
            Ok(ref x) if !x.is_empty() => {
                println!("TEST");

                match test_step {
                    Some(ref test_command) => {
                        run_test(&test_command, &test_cmd_args)
                    }
                    _ => {
                        println!("The test step has not been specified, doing nothing.")
                    }
                }
            }
            Ok(_) => (),
            Err(e) => println!("watch error: {:?}", e),
        }
    });

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.

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
                // todo now: Instead of directly handling each individual event,
                // they should be stored in a collection and handled in a single
                // go, as soon as the 'delay' period from the first event time
                // has passed.
                let mut collected_events = collected_events.lock().unwrap();

                handle_watch_events(
                    &event,
                    &matching_files,
                    &mut collected_events,
                );
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn get_test_step(matches: &ArgMatches) -> Option<String> {
    return matches.value_of("TEST STEP").map(|s| s.to_owned());
}

fn get_test_cmd_args(matches: &ArgMatches) -> Option<Vec<String>> {
    let test_cmd_args = matches.values_of("TEST COMMAND ARGS");

    return match test_cmd_args {
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
}

fn handle_watch_events(
    event: &DebouncedEvent,
    matching_files: &RegexSet,
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
        }
        None => (),
    }
}

fn run_test(test_command: &str, test_cmd_args: &Option<Vec<String>>) {
    let mut command = Command::new(test_command);

    match test_cmd_args {
        Some(ref args) => {
            command.args(args);
        }
        None => (),
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
