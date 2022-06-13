# TCR (test && commit || revert) Tool

TCR is a command-line tool that helps you run the TCR (test && commit || revert) development workflow. You can read Kent Beck's introductory [article on TCR](https://medium.com/@kentbeck_7670/test-commit-revert-870bbd756864). He also has a [miniseries of videos](https://www.youtube.com/watch?v=tnO2Mos0RjU) showing how he does TCR.

## What does this tool do?

Simply put, the tool watches for any changes in your source code and once changes are detected, the tool runs a test command (provided by you). If the test is successful, the tool commits the code to the git repository. On the other hand, if the test fails, the tool reverts all of your local git changes.

Although TCR Tool is written in Rust, it is designed to be used by any language platform that supports executing tests from the command line.

## How to use it?

### A word of warning

> :warning: be careful when trying out the tool for the first time. Make sure you have all your source code committed. If the test fails, the tool will execute `git reset --hard` command, so you may lose your valuables. Also, if the test succeeds, the tool executes the following command:

```cmd
git add .
git commit -a -q -m works
```

> ...so it will add everything that is not git-ignored to the repository and commit it.

### Example usage

Here's an example Windows script I use to run TCR in one of my Python projects (split into multiple lines just for clarity):

```cmd
tcr --file-pattern .*.py$ --test pytest 
    -- 
    --quiet -p no:sugar --disable-warnings 
    %~1
```

Let's disect the above command line:

- `--file-pattern .*.py$` argument instructs TCR tool to only look for Python source code files when detecting changes.
- `--test pytest` tells the tool to run `pytest` as the testing tool (once the source code changes have been detected).
- `--` is the delimiter between TCR tool's command line arguments and those that will be forwarded to `pytest`.
- `--quiet -p no:sugar --disable-warnings` are the arguments that are sent to `pytest` (and not relevant to TCR tool itself).
- `%~1`: is just a placeholder for the name of the Python file containing tests I want to be run (also just forwarded to `pytest`).

Since no other parameters were specified, the TCR tool in this case defaults to the following:

- the root path of the source code to be watched is the current directory.
- the delay between the first detected source file change and actually running the test is 1 second. This delay setting can be useful if you need to save several files before the tests are run (although I personally just use "Save all" keyboard shortcut to save all files in one go).

### Command-line parameters

You can get the command-line help by running `tcr --help` command:

```cmd
USAGE:
    tcr.exe [OPTIONS] [TEST COMMAND ARGS]...

ARGS:
    <TEST COMMAND ARGS>...

OPTIONS:
    -t, --test <TEST STEP>
            The command to run as a test step. If not specified, only a warning will be printedduring the test step.

    -p, --path <PATH>
            The path to watch for file changes. [default: .]

    -f, --file-pattern <FILE PATTERN>
            The regex file pattern which changed/created/deleted files must match to trigger the test step. [default: .*.rs]

    -d, --delay <DELAY>
            The delay (in milliseconds) between the first detected file change and running the test step. [default: 1000]

    -h, --help
            Print help information

    -V, --version
            Print version information
```

## Some notes

- When the tool reverts the code after a failed test, it does not rerun the tests (even though the code on the disk has been changed by the revert). This is on purpose: we want to be able to see the output of the last failed test.
