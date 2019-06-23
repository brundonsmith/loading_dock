
use std::path::{PathBuf};
use std::ops::{Deref};
use std::sync::{Mutex};
use std::time::{Duration};
use std::collections::{HashMap,HashSet};

use reqwest::Client;
use url::form_urlencoded;
extern crate reqwest;

// internal
use crate::serialization;

lazy_static! {
    static ref HTTP: Client = Client::new();
}

pub fn greet_contact(file_timestamps: &'static Mutex<HashMap<PathBuf,Duration>>, other_nodes: &'static Mutex<HashSet<String>>, my_port: &str, address: &str) {
    println!("greet_contact {} =>", address);

    let mut res = 
        HTTP.post(&(String::from("http://") + address + "/greet/" + my_port))
            .send()
            .unwrap();

    let others = res.text().unwrap();
    let mut locked = other_nodes.lock().unwrap();
    
    locked.insert(String::from(address));
    serialization::deserialize_other_nodes(&others).iter()
        .for_each(|other| {
            locked.insert(other.to_owned());
        });

    println!("other_nodes: {:?}", locked);
}

pub fn publish_file(other_nodes: &'static Mutex<HashSet<String>>, relative_path: &PathBuf, file_contents: &Vec<u8>) {
    let string = relative_path.to_string_lossy();
    let encoding: Vec<&str> = form_urlencoded::byte_serialize(string.as_bytes()).collect();
    let encoded: String = encoding.into_iter().collect();

    Deref::deref(&other_nodes.lock().unwrap()).iter()
        .for_each(|other| {
            println!("publish_file {} =>", &other);
            HTTP.post(&(String::from("http://") + &other + "/file/" + &encoded))
                .body(file_contents.to_owned())
                .send()
                .unwrap();
        })
}
