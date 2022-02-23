extern crate notify;

use clap::Parser;
use notify::DebouncedEvent::{Create, Remove, Rename, Write};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use regex::RegexSet;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Parser, Debug)]
struct Args {
    /// path to watch
    #[clap(short, long, default_value = ".")]
    path: String,

    /// pattern of files to watch
    #[clap(short, long, default_value = r".*.\.rs$")]
    file_pattern: String,

    /// delay (in milliseconds) before executing the tests
    #[clap(short, long, default_value = "1000")]
    delay: u64,
}

fn main() {
    let args = Args::parse();

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
            Ok(event) => handle_watch_event(&event, &matching_files),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn handle_watch_event(event: &DebouncedEvent, matching_files: &RegexSet) {
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
        println!("{:?}", event_data.unwrap())
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
