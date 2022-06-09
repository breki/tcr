extern crate notify;

mod args;
mod git;
mod paths;
mod testing;
mod watch;

use notify::{raw_watcher, RawEvent, RecursiveMode, Watcher};
use regex::RegexSet;
use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use watch::SourceCodeUpdateEvent;

fn collect_watch_events(
    files_watch_enabled: Arc<Mutex<bool>>,
    rx_watch_events: Receiver<RawEvent>,
    collected_events: Arc<Mutex<HashSet<watch::SourceCodeUpdateEvent>>>,
    tx_watch_events_starter: Sender<u32>,
    matching_files: RegexSet,
) {
    loop {
        match (rx_watch_events.recv(), *files_watch_enabled.lock().unwrap()) {
            (Ok(event), true) => match event.path {
                Some(path) => {
                    let mut collected_events = collected_events.lock().unwrap();

                    match watch::filter_interesting_event(
                        &path,
                        &matching_files,
                    ) {
                        Some(event_data) => {
                            if collected_events.len() == 0 {
                                // clear the terminal
                                print!("\x1B[2J");
                                println!("DETECTED CHANGES:");
                                tx_watch_events_starter.send(1).unwrap();
                            }

                            if !collected_events.contains(&event_data) {
                                println!("{}", event_data);
                            }

                            collected_events.insert(event_data);
                        }
                        None => (),
                    }
                }
                None => (),
            },
            (Err(e), _) => println!("watch error: {:?}", e),
            (_, false) => (),
        }
    }
}

fn run_tests_on_files_update(
    files_watch_enabled: Arc<Mutex<bool>>,
    rx_watch_events_starter: Receiver<u32>,
    collected_events: Arc<Mutex<HashSet<SourceCodeUpdateEvent>>>,
    delay: Duration,
    test_step: Option<String>,
    test_cmd_args: Option<Vec<String>>,
) {
    loop {
        match rx_watch_events_starter.recv() {
            Ok(1) => {
                thread::sleep(delay);

                {
                    let mut collected_events = collected_events.lock().unwrap();
                    collected_events.clear();
                }

                match test_step {
                    Some(ref test_command) => {
                        println!("RUNNING TESTS...");
                        match testing::run_test(&test_command, &test_cmd_args) {
                            testing::TestsResult::SUCCESS => {
                                println!("TESTS SUCCEEDED");
                                git::git_commit();
                            }
                            testing::TestsResult::FAILURE => {
                                println!("TESTS FAILED");

                                {
                                    let mut files_watch_enabled =
                                        files_watch_enabled.lock().unwrap();
                                    *files_watch_enabled = false;
                                }

                                git::git_revert();

                                {
                                    let mut files_watch_enabled =
                                        files_watch_enabled.lock().unwrap();
                                    *files_watch_enabled = true;
                                }
                            }
                        }
                    }
                    _ => {
                        println!(
                            "The test step has not been specified, \
                            doing nothing."
                        )
                    }
                }

                println!("----------------------------------------");
            }
            _ => (),
        }
    }
}

fn main() {
    let matches = args::parse_args();

    let test_step = args::get_test_step(&matches);
    let test_cmd_args = args::get_test_cmd_args(&matches);
    let file_pattern = matches.value_of("FILE PATTERN").unwrap();

    let matching_files = RegexSet::new(&[file_pattern]).unwrap();

    let delay_ms = matches.value_of("delay").unwrap().parse::<u64>().unwrap();
    let delay = Duration::from_millis(delay_ms);

    // Create a channel to receive the events.
    let (tx_watch_events_starter, rx_watch_events_starter) = channel();

    let files_watch_enabled: Arc<Mutex<bool>> = Arc::new(Mutex::new(true));
    let files_watch_enabled_clone = Arc::clone(&files_watch_enabled);

    let collected_events: Arc<Mutex<HashSet<watch::SourceCodeUpdateEvent>>> =
        Arc::new(Mutex::new(HashSet::new()));
    let collected_events_clone = Arc::clone(&collected_events);
    thread::spawn(move || {
        run_tests_on_files_update(
            files_watch_enabled,
            rx_watch_events_starter,
            collected_events_clone,
            delay,
            test_step,
            test_cmd_args,
        )
    });

    // Create a channel to receive the events.
    let (tx_watch_events, rx_watch_events) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.

    let mut watcher = raw_watcher(tx_watch_events).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    let path_to_watch = matches.value_of("path").unwrap();

    watcher
        .watch(path_to_watch, RecursiveMode::Recursive)
        .unwrap();

    collect_watch_events(
        files_watch_enabled_clone,
        rx_watch_events,
        collected_events,
        tx_watch_events_starter,
        matching_files,
    );
}
