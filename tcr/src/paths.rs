use regex::RegexSet;
use std::path::Path;

pub fn is_path_matched(path: &Path, matching_files: &RegexSet) -> bool {
    return matching_files.is_match(&path_to_str(&path));
}

pub fn path_to_str(path: &Path) -> String {
    let os_path_str = path.as_os_str();
    os_path_str.to_str().unwrap().to_string()
}
