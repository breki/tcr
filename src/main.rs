extern crate notify;

use notify::DebouncedEvent::NoticeWrite;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use regex::RegexSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

fn main() {
    let matching_files = RegexSet::new(&[r".*.\.rs$"]).unwrap();

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let watch_period = Duration::from_secs(2);

    let mut watcher = watcher(tx, watch_period).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher
        .watch("d:\\src\\tcr", RecursiveMode::Recursive)
        .unwrap();

    loop {
        match rx.recv() {
            Ok(event) => handle_watch_event(event, &matching_files),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn handle_watch_event(event: DebouncedEvent, matching_files: &RegexSet) {
    let event_path: Option<String> = match event {
        NoticeWrite(path) => extract_event_path(path, matching_files),
        _ => None,
    };

    if event_path.is_some() {
        println!("{:?} written", event_path.unwrap())
    }
}

fn extract_event_path(
    path: PathBuf,
    matching_files: &RegexSet,
) -> Option<String> {
    if is_path_matched(&path, matching_files) {
        Some(path_to_str(&path))
    } else {
        None
    }
}

fn is_path_matched(path: &PathBuf, matching_files: &RegexSet) -> bool {
    return matching_files.is_match(&path_to_str(&path));
}

fn path_to_str(path: &Path) -> String {
    let os_path_str = path.as_os_str();
    os_path_str.to_str().unwrap().to_string()
}
