# TCR (test &amp;&amp; commit || revert) Tool

TCR is a command-line tool that helps you run the TCR (test &amp;&amp; commit || revert) development workflow (see Kent Beck's introductory [article on TCR](https://medium.com/@kentbeck_7670/test-commit-revert-870bbd756864)).

## What does this tool do?

Simply put, the tool watches for any changes in your code and once some changes are detected, the tool runs a test command (provided by you). If the test is successful, the tool commits the code to the git repository. On the other hand, if the test fails, it reverts all local git changes.

Although TCR Tool is written in Rust, it is designed to be used by any language platform that supports executing tests from the command line.

## How to use it?

Here's an example Windows script I use to run TCR in one of my Python projects:

```cmd
tcr --file-pattern .*.py$ --test pytest 
    -- 
    --quiet -p no:sugar --disable-warnings 
    %~1
```

Let's disect the above command line:
- `--file-pattern .*.py$` argument instructs TCR tool to limit the file changes watching to Python source code files.
- `--test pytest` tells the tool to run `pytest` as the testing tool (once the source code changes have been detected).
- `--` is the delimiter between TCR tool's command line arguments and those that will be forwarded to `pytest`.
- `--quiet -p no:sugar --disable-warnings` are the arguments that are sent to `pytest`.
- `%~1`: is just a placeholder for the name of the Python file containing tests I want to be run.

Since no other parameters were specified, the TCR tool in this case defaults to the following:
- the root path of the source code to be watched is the current directory.
- the delay between the first detected source file change and actually running the test is 1 second.

**WARNING**: be careful when trying out the tool for the first time. Make sure you have all your source code committed. If the test fails, the tool will execute `git reset --hard` command, so you may lose your valuables. Also, if the test succeeds, the tool executes
```
git add .
git commit -a -q -m works
```
commands, so it will add everything that is not git-ignored to the repository and commit it.
