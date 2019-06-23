use std::env;
use std::path::{PathBuf};
use std::sync::{Mutex};
use std::sync::mpsc::channel;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::{HashMap,HashSet};
use std::thread;
use std::str;
use std::fs::File;
use std::io::{Read};

extern crate notify;
use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
extern crate pathdiff;
extern crate iron;
use iron::prelude::*;
extern crate router;
use router::Router;

#[macro_use]
extern crate lazy_static;


// internal
mod server;
mod requests;
mod serialization;

// 1. Check network for other instances (given port from CLI, or default port)
// 2. If no others found...
//      a. Record time stamps of all files in local directory
//      b. Listen for connections from other, new instances, 
// 3. If others found...
//      a. Get their lists of known instances
//      b. Get their lists of file names and timestamps
//          i.  For each new file on another instance, pull it
//          ii. For each file present locally that another instance doesn't have, push it
// 4. Listen for local file changes. On change, push to all other known instances.

const DEFAULT_PORT: &str = "9123";

/* State */
lazy_static! {
    static ref FILE_TIMESTAMPS: Mutex<HashMap<PathBuf,Duration>> = Mutex::new(HashMap::new());
    static ref OTHER_NODES: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

fn main() {
    print!("main()\n");

    /* CLI Args */
    let args: Vec<String> = env::args().collect();
    let dir = PathBuf::from(args.get(1).unwrap());
    let port = String::from(match args.get(2) {
        Some(p) => p,
        None => DEFAULT_PORT
    });
    let contact = args.get(3);
    

    /* Start HTTP server */
    let mut router = Router::new();
    server::init_server(&mut router, &FILE_TIMESTAMPS, &OTHER_NODES, dir.to_owned());

    /* Greet */
    match contact {
        Some(other) => requests::greet_contact(&FILE_TIMESTAMPS, &OTHER_NODES, &port, other),
        None => {}
    };

    thread::spawn(move || {
        print!("Listening on localhost:{}\n", &port);
        Iron::new(router).http("localhost:".to_owned() + &port).unwrap();
    });

    /* Watching */

    // Create a channel to receive the events.
    let (sender, receiver) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(sender, Duration::from_secs(1)).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    print!("{}\n", dir.to_str().unwrap());
    watcher.watch(&dir, RecursiveMode::Recursive).unwrap();

    loop {
        match receiver.recv() {
            Ok(event) => {
                print!("{:?}\n", event);
                match event {
                    | DebouncedEvent::NoticeWrite(file_path)
                    | DebouncedEvent::NoticeRemove(file_path)
                    | DebouncedEvent::Create(file_path)
                    | DebouncedEvent::Write(file_path)
                    | DebouncedEvent::Chmod(file_path)
                    | DebouncedEvent::Remove(file_path)
                    | DebouncedEvent::Rename(_, file_path) => handle_file_change(&dir, &FILE_TIMESTAMPS, &OTHER_NODES, &file_path),
                    _ => (),
                };
            },
            Err(e) => print!("watch error: {:?}\n", e),
        }
    }
}

/*
fn get_watch_dir() -> PathBuf {
    env::current_dir().unwrap().join("dir-test")
}
*/

fn get_timestamp() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

fn handle_file_change(dir: &PathBuf, file_timestamps: &'static Mutex<HashMap<PathBuf,Duration>>, other_nodes: &'static Mutex<HashSet<String>>, file_path: &PathBuf) {
    let relative_path = pathdiff::diff_paths(&file_path, &dir).unwrap();

    // log timestamp
    file_timestamps.lock().unwrap().insert(relative_path.to_owned(), get_timestamp());

    // load file
    let mut file = File::open(file_path).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf).unwrap();

    // send file
    requests::publish_file(other_nodes, &relative_path, &buf);
}
