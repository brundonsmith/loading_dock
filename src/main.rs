extern crate notify;
extern crate pathdiff;
extern crate iron;
extern crate router;
extern crate reqwest;

#[macro_use]
extern crate lazy_static;

use std::env;
use std::path::{PathBuf};
use std::sync::{Mutex, Arc};
use std::sync::mpsc::channel;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};

use reqwest::Client;

mod server;
use server::init_server;


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

/* Primary state */
lazy_static! {
    static ref FILE_TIMESTAMPS: Arc<Mutex<HashMap<PathBuf,Duration>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref OTHER_NODES: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
}

fn main() {
    print!("Hello world!\n");

    /* CLI Args */
    // TODO: https://stackoverflow.com/questions/15619320/how-to-access-command-line-parameters
    
    let args: Vec<String> = env::args().collect();

    let port = match args.get(1) {
        Some(p) => p,
        None => DEFAULT_PORT
    };
    match args.get(2) {
        Some(other) => greet_contact(&OTHER_NODES, &port, other),
        None => {}
    };


    /* Start HTTP server */
    init_server(port, &FILE_TIMESTAMPS, &OTHER_NODES);


    /* Watching */

    // Create a channel to receive the events.
    let (sender, receiver) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(sender, Duration::from_secs(1)).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(get_watch_dir(), RecursiveMode::Recursive).unwrap();

    loop {
        match receiver.recv() {
            Ok(event) => {
                print!("{:?}\n", event);
                match event {
                    | DebouncedEvent::NoticeWrite(path)
                    | DebouncedEvent::NoticeRemove(path)
                    | DebouncedEvent::Create(path)
                    | DebouncedEvent::Write(path)
                    | DebouncedEvent::Chmod(path)
                    | DebouncedEvent::Remove(path) => log_change(&FILE_TIMESTAMPS, path),
                      DebouncedEvent::Rename(_path1, path2) => log_change(&FILE_TIMESTAMPS, path2),
                    _ => (),
                };
            },
            Err(e) => print!("watch error: {:?}\n", e),
        }
    }
}

fn get_watch_dir() -> PathBuf {
    env::current_dir().unwrap().join("/dir-test")
}

fn get_timestamp() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

fn log_change(map: &'static Arc<Mutex<HashMap<PathBuf,Duration>>>, path: PathBuf) {
    let relative_path = pathdiff::diff_paths(&path, &get_watch_dir()).unwrap();
    map.lock().unwrap().insert(relative_path, get_timestamp());
    print!("Change logged to: {:?}\n", map);
}

fn greet_contact(other_nodes: &'static Arc<Mutex<Vec<String>>>, my_port: &str, address: &str) {
    print!("Greeting {}\n", address);
    Client::new()
        .post(&(String::from("http://") + address + "/greet/" + my_port))
        .send()
        .and_then(|_res| {
            other_nodes.lock().unwrap().push(String::from(address));
            return Ok(());
        })
        .unwrap();
}

