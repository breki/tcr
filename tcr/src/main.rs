extern crate notify;

mod args;
mod paths;
mod testing;
mod watch;

use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use regex::RegexSet;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use watch::SourceCodeUpdateEvent;

fn collect_watch_events(
    rx_watch_events: Receiver<DebouncedEvent>,
    collected_events: Arc<Mutex<Vec<watch::SourceCodeUpdateEvent>>>,
    tx_watch_events_starter: Sender<u32>,
    matching_files: RegexSet,
) {
    loop {
        match rx_watch_events.recv() {
            Ok(event) => {
                let mut collected_events = collected_events.lock().unwrap();

                match watch::filter_interesting_event(&event, &matching_files) {
                    Some(event_data) => {
                        println!("{}", event_data);
                        if collected_events.len() == 0 {
                            tx_watch_events_starter.send(1).unwrap();
                        }
                        collected_events.push(event_data);
                    }
                    None => (),
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn run_tests_on_files_update(
    rx_watch_events_starter: Receiver<u32>,
    collected_events: Arc<Mutex<Vec<SourceCodeUpdateEvent>>>,
    watch_period: Duration,
    test_step: Option<String>,
    test_cmd_args: Option<Vec<String>>,
) {
    loop {
        match rx_watch_events_starter.recv() {
            Ok(1) => {
                thread::sleep(watch_period);

                {
                    let mut collected_events = collected_events.lock().unwrap();
                    collected_events.clear();
                }

                match test_step {
                    Some(ref test_command) => {
                        match testing::run_test(&test_command, &test_cmd_args) {
                            testing::TestsResult::SUCCESS => {
                                println!("TESTS SUCCEEDED");
                            }
                            testing::TestsResult::FAILURE => {
                                println!("TESTS FAILED");
                            }
                        }
                    }
                    _ => {
                        println!(
                            "The test step has not been specified, doing nothing.")
                    }
                }
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

    let delay = matches.value_of("delay").unwrap().parse::<u64>().unwrap();
    let watch_period = Duration::from_millis(delay);

    // Create a channel to receive the events.
    let (tx_watch_events_starter, rx_watch_events_starter) = channel();

    let collected_events: Arc<Mutex<Vec<watch::SourceCodeUpdateEvent>>> =
        Arc::new(Mutex::new(Vec::new()));
    let collected_events_clone = Arc::clone(&collected_events);
    thread::spawn(move || {
        run_tests_on_files_update(
            rx_watch_events_starter,
            collected_events_clone,
            watch_period,
            test_step,
            test_cmd_args,
        )
    });

    // Create a channel to receive the events.
    let (tx_watch_events, rx_watch_events) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.

    let delay = Duration::from_millis(500);
    let mut watcher = watcher(tx_watch_events, delay).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    let path_to_watch = matches.value_of("path").unwrap();

    watcher
        .watch(path_to_watch, RecursiveMode::Recursive)
        .unwrap();

    collect_watch_events(
        rx_watch_events,
        collected_events,
        tx_watch_events_starter,
        matching_files,
    );
}
