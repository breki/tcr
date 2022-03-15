use notify::DebouncedEvent;
use notify::DebouncedEvent::{Create, Remove, Rename, Write};
use regex::RegexSet;
use std::fmt;
use std::path::Path;

use crate::paths;

pub struct SourceCodeUpdateEvent {
    path: String,
    event_type: String,
}

impl fmt::Display for SourceCodeUpdateEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.event_type, self.path)
    }
}

pub fn handle_watch_events(
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

fn extract_event_data(
    event_desc: &str,
    path: &Path,
    matching_files: &RegexSet,
) -> Option<SourceCodeUpdateEvent> {
    if paths::is_path_matched(&path, matching_files) {
        Some(SourceCodeUpdateEvent {
            path: paths::path_to_str(&path),
            event_type: event_desc.to_string(),
        })
    } else {
        None
    }
}
