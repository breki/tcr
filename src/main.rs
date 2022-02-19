extern crate notify;

use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

fn main() {
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let watch_period = Duration::from_secs(2);

    let mut watcher = watcher(tx, watch_period).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes. 
    watcher.watch("d:\\src\\tcr", RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() { 
           Ok(event) => println!("{:?}", event),
           Err(e) => println!("watch error: {:?}", e),
        }
    }
}
