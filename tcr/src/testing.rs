use std::io;
use std::io::Write as IoWrite;
use std::process::Command;

pub enum TestsResult {
    SUCCESS,
    FAILURE,
}

pub fn run_test(
    test_command: &str,
    test_cmd_args: &Option<Vec<String>>,
) -> TestsResult {
    let mut command = Command::new(test_command);

    match test_cmd_args {
        Some(ref args) => {
            command.args(args);
        }
        None => (),
    }

    let output = command.output().expect("failed to execute the test step");
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    match command.status() {
        Ok(status) => {
            if status.success() {
                TestsResult::SUCCESS
            } else {
                TestsResult::FAILURE
            }
        }
        Err(e) => {
            println!("failed to execute the test step: {}", e);
            TestsResult::FAILURE
        }
    }
}
