
use std::path::{PathBuf};
use std::time::{Duration};
use std::collections::{HashMap,HashSet};

const DELIMITER: &str = "|";
const KEY_VAL_SEPARATOR: &str = "\t";

struct Interspersed<T: Copy,I: Iterator<Item=T>> {
    source: I,
    delimiter: T,
    delimit: bool,
}

impl<T: Copy, I: Iterator<Item=T>> Interspersed<T,I> {
    fn new(source: I, delimiter: T) -> Interspersed<T,I> {
        Interspersed { source, delimiter, delimit: false }
    }
}

impl<T: Copy,I: Iterator<Item=T>> Iterator for Interspersed<T,I> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        
        let res = if self.delimit {
            Option::Some(self.delimiter)
        } else {
            self.source.next()
        };

        self.delimit = !self.delimit;

        return res;
    }
}

fn foo() {
    Interspersed::new(vec![1, 2, 3, 4, 5].iter(), &0).collect::<Vec<&u32>>();
}

pub fn serialize_file_timestamps(file_timestamps: &HashMap<PathBuf,Duration>) -> String {
    let mut result = String::new();

    file_timestamps.iter()
        .for_each(|(file, timestamp)| {
            result.push_str(DELIMITER);
            result.push_str(&file.to_string_lossy());
            result.push_str(KEY_VAL_SEPARATOR);
            result.push_str(timestamp.as_millis().to_string().as_ref());
        });

    return result;
}

pub fn deserialize_file_timestamps(string: &str) -> HashMap<PathBuf,Duration> {
    return string.split(DELIMITER)
        .filter(|&seg| seg != "")
        .map(|entry| {
            let kv = entry.split(KEY_VAL_SEPARATOR).collect::<Vec<&str>>();
            let path = PathBuf::from(kv[0]);
            let timestamp = Duration::from_millis(kv[1].parse::<u64>().unwrap());
            return (path, timestamp)
        })
        .collect::<HashMap<PathBuf,Duration>>();
}


pub fn serialize_other_nodes(other_nodes: &HashSet<String>) -> String {
    let mut response = String::new();

    other_nodes.iter()
        .for_each(|address| {
            response.push_str(DELIMITER);
            response.push_str(&address);
        });

    return response;
}

pub fn deserialize_other_nodes(string: &str) -> HashSet<String> {
    return string.split(DELIMITER)
        .filter(|&seg| seg != "")
        .map(|seg| String::from(seg))
        .collect::<HashSet<String>>();
}