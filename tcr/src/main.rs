extern crate notify;

use clap::Parser;
use notify::DebouncedEvent::{Create, Remove, Rename, Write};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use regex::RegexSet;
use std::io;
use std::io::Write as IoWrite;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Parser, Debug)]
#[clap(name = "tcr")]
#[clap(author = "Igor Brejc")]
#[clap(version = "1.0")]
#[clap(about = "Helps run 'test && commit || revert' workflow")]
struct Args {
    /// Program or script to run as a test step. Does nothing if not specified.
    #[clap(short, long)]
    test: Option<String>,

    /// Path to watch.
    #[clap(short, long, default_value = ".")]
    path: String,

    /// Pattern of files to watch.
    #[clap(short, long, default_value = r".*.\.rs$")]
    file_pattern: String,

    /// Delay (in milliseconds) before executing the tests.
    #[clap(short, long, default_value = "1000")]
    delay: u64,
}

fn main() {
    let args = Args::parse();

    let test_step = args.test;

    let matching_files = RegexSet::new(&[args.file_pattern]).unwrap();

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let watch_period = Duration::from_millis(args.delay);

    let mut watcher = watcher(tx, watch_period).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(args.path, RecursiveMode::Recursive).unwrap();

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
    test_step: &Option<String>,
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
        // todo now: redesign the test step as an external command
        // that needs to be run + additional arguments provided after the '--'
        // argument
        println!("TEST");

        match test_step {
            Some(test_command) => {
                let output = Command::new(test_command)
                    .output()
                    .expect("failed to execute process");

                io::stdout().write_all(&output.stdout).unwrap();
                io::stderr().write_all(&output.stderr).unwrap();

                // todo now: check the test exit code.
                // In case of success, run commit.
                // In case of failure, run reset.
            }
            None => {
                println!("The test step has not been specified, doing nothing.")
            }
        }
    }
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
