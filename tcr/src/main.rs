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
                // todo now: Instead of directly handling each individual event,
                // they should be stored in a collection and handled in a single
                // go, as soon as the 'delay' period from the first event time
                // has passed.
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

fn group_events(
    rx_watch_events_starter: Receiver<u32>,
    collected_events_thread: Arc<Mutex<Vec<SourceCodeUpdateEvent>>>,
    tx_collected_events: Sender<Vec<SourceCodeUpdateEvent>>,
    watch_period: Duration,
) {
    loop {
        match rx_watch_events_starter.recv() {
            Ok(1) => {
                println!("DETECTED FIRST CHANGE EVENT...");
                thread::sleep(watch_period);

                let mut collected_events =
                    collected_events_thread.lock().unwrap();

                let mut events_to_process: Vec<SourceCodeUpdateEvent> =
                    Vec::new();
                while collected_events.len() > 0 {
                    let event = collected_events.pop().unwrap();
                    events_to_process.push(event);
                }
                match tx_collected_events.send(events_to_process) {
                    Ok(_) => (),
                    Err(e) => {
                        println!(
                            "Error sending source code change events: {}",
                            e
                        );
                    }
                }
            }
            _ => (),
        }
    }
}

fn run_tests_on_watch(
    rx_collected_events: Receiver<Vec<SourceCodeUpdateEvent>>,
    test_step: Option<String>,
    test_cmd_args: Option<Vec<String>>,
) {
    loop {
        match rx_collected_events.recv() {
            Ok(ref x) if !x.is_empty() => {
                println!("TEST");

                match test_step {
                    Some(ref test_command) => {
                        testing::run_test(&test_command, &test_cmd_args)
                    }
                    _ => {
                        println!("The test step has not been specified, doing nothing.")
                    }
                }
            }
            Ok(_) => (),
            Err(e) => println!("watch error: {:?}", e),
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

    let collected_events: Arc<Mutex<Vec<watch::SourceCodeUpdateEvent>>> =
        Arc::new(Mutex::new(Vec::new()));
    let (tx_collected_events, rx_collected_events) =
        channel::<Vec<SourceCodeUpdateEvent>>();

    // Create a channel to receive the events.
    let (tx_watch_events_starter, rx_watch_events_starter) = channel();

    let collected_events_thread = Arc::clone(&collected_events);
    thread::spawn(move || {
        group_events(
            rx_watch_events_starter,
            collected_events_thread,
            tx_collected_events,
            watch_period,
        )
    });
    thread::spawn(move || {
        run_tests_on_watch(rx_collected_events, test_step, test_cmd_args)
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
