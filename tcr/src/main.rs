extern crate notify;

mod args;
mod paths;
mod testing;
mod watch;

use notify::{watcher, RecursiveMode, Watcher};
use regex::RegexSet;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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
    let (tx_collected_events, rx_collected_events) = channel();

    let collected_events_thread = Arc::clone(&collected_events);
    thread::spawn(move || loop {
        {
            let mut collected_events = collected_events_thread.lock().unwrap();

            let mut events_to_process: Vec<watch::SourceCodeUpdateEvent> =
                Vec::new();
            while collected_events.len() > 0 {
                let event = collected_events.pop().unwrap();
                events_to_process.push(event);
            }
            match tx_collected_events.send(events_to_process) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error sending source code change events: {}", e);
                }
            }
        }

        println!("SLEEPER THREAD");
        thread::sleep(watch_period);
    });

    thread::spawn(move || loop {
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
    });

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.

    let mut watcher = watcher(tx, watch_period).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    let path_to_watch = matches.value_of("path").unwrap();

    watcher
        .watch(path_to_watch, RecursiveMode::Recursive)
        .unwrap();

    loop {
        match rx.recv() {
            Ok(event) => {
                // todo now: Instead of directly handling each individual event,
                // they should be stored in a collection and handled in a single
                // go, as soon as the 'delay' period from the first event time
                // has passed.
                let mut collected_events = collected_events.lock().unwrap();

                watch::handle_watch_events(
                    &event,
                    &matching_files,
                    &mut collected_events,
                );
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
