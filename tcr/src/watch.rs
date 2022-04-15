use regex::RegexSet;
use std::fmt;
use std::path::Path;

use crate::paths;

pub struct SourceCodeUpdateEvent {
    path: String,
}

impl fmt::Display for SourceCodeUpdateEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

pub fn filter_interesting_event(
    path: &Path,
    matching_files: &RegexSet,
) -> Option<SourceCodeUpdateEvent> {
    // println!("{:?}", path);
    if paths::is_path_matched(&path, matching_files) {
        Some(SourceCodeUpdateEvent {
            path: paths::path_to_str(&path),
        })
    } else {
        None
    }
}
