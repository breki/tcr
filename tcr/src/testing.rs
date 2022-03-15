use std::io;
use std::io::Write as IoWrite;
use std::process::Command;

pub fn run_test(test_command: &str, test_cmd_args: &Option<Vec<String>>) {
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
