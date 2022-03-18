use std::process::Command;

pub fn git_commit() {
    let mut command = Command::new("git");
    command.args(&["add", "."]);
    command
        .output()
        .expect("failed to execute the git add step");

    command = Command::new("git");
    command.args(&["commit", "-a", "-q", "-m", "works"]);
    command
        .output()
        .expect("failed to execute the git commit step");
    println!("COMMIT")
}

pub fn git_revert() {
    let mut command = Command::new("git");
    command.args(&["reset", "--hard"]);
    command
        .output()
        .expect("failed to execute the git reset step");
    println!("REVERT");
}
