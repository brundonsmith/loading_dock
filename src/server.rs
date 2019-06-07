use std::path::{PathBuf};
use std::sync::{Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::fs::File;
use std::io::{Read, Write};
use std::ops::{Deref};
use std::net::IpAddr;
use std::collections::{HashMap, HashSet};

extern crate notify;
extern crate pathdiff;
extern crate iron;
use iron::prelude::*;
use iron::status;
use router::Router;
extern crate router;
extern crate url;
use url::form_urlencoded;


pub fn init_server(router: &mut Router, file_timestamps: &'static Mutex<HashMap<PathBuf,Duration>>, other_nodes: &'static Mutex<HashSet<String>>, dir: PathBuf) {

    router.get("/", |_req: &mut Request| -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Hello world!")))
    }, "test");

    router.post("/greet/:remote_port", move |req: &mut Request| -> IronResult<Response> {
        
        // get remote address and port
        let remote_port = get_param(req, "remote_port");
        let other_addr: String = get_ip(req) + ":" + remote_port;
        println!("POST /greet/{}", &other_addr);

        // record new node
        let mut locked = other_nodes.lock().unwrap();
        let response = serialize_nodes(&locked);

        locked.insert(other_addr);

        print!("other_nodes: {:?}\n", locked);
        
        return Ok(Response::with((status::Ok, response)));
    }, "greet");

    // Pull file <-
    let dir2 = dir.to_owned();
    router.get("/file/:file_path", move |req: &mut Request| -> IronResult<Response> {

        // get file path
        let file_path = get_param(req, "file_path");
        let parse: Vec<(String, String)> = form_urlencoded::parse(file_path.as_bytes()).into_owned().collect();
        let file_path = parse[0].0.to_owned();

        // get file contents
        let mut file = File::open(dir2.join(file_path)).unwrap();
        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        
        return Ok(Response::with((status::Ok, buf)));
    }, "get_file");

    // Push file change ->
    let dir3 = dir.to_owned();
    router.post("/file/:file_path", move |req: &mut Request| -> IronResult<Response> {

        // get file path
        let file_path = get_param(req, "file_path");
        let parse: Vec<(String, String)> = form_urlencoded::parse(file_path.as_bytes()).into_owned().collect();
        let file_path = parse[0].0.to_owned();
        println!("POST /file/{}", &file_path);

        // get request body
        let mut buf: Vec<u8> = Vec::new();
        req.body.read_to_end(&mut buf).unwrap();

        // write to file
        let path = dir3.join(file_path);
        let mut file = match File::open(&path) {
            Ok(f) => f,
            Err(_) => File::create(&path).unwrap()
        };
        file.write(&buf).unwrap();
        
        // record new timestamp
        let mut locked = file_timestamps.lock().unwrap();
        locked.insert(PathBuf::new(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap());
        
        return Ok(Response::with(status::Ok));
    }, "post_file");

    // Pull files list <-
    router.get("/all-files", move |_req: &mut Request| -> IronResult<Response> {
        let mut response = String::new();

        // serialize map
        let mut_g = file_timestamps.lock().unwrap();
        let locked = Deref::deref(&mut_g);
        for (file, timestamp) in locked {
            response.push_str("\n");
            response.push_str(&file.to_string_lossy());
            response.push_str("|");
            response.push_str(timestamp.as_millis().to_string().as_ref());
        }

        return Ok(Response::with((status::Ok, response)));
    }, "all_files");

    // Pull nodes list <-
    router.get("/other-nodes", move |_req: &mut Request| -> IronResult<Response> {
        let mut response = String::new();

        // serialize list
        let mut_g = other_nodes.lock().unwrap();
        let locked = Deref::deref(&mut_g);
        for address in locked {
            response.push_str("\n");
            response.push_str(&address);
        }

        return Ok(Response::with((status::Ok, response)));
    }, "other_nodes");

}

fn get_param<'a>(req: &'a Request, name: &str) -> &'a str {
    req.extensions.get::<Router>().unwrap().find(name).unwrap()
}

fn get_ip(req: &Request) -> String {
    match req.remote_addr.ip() {
        IpAddr::V4(addr) => addr.to_string(),
        IpAddr::V6(addr) => String::from("[") + &addr.to_string() + "]"
    }
}

fn serialize_nodes(nodes: &HashSet<String>) -> String {
    let mut res = String::new();

    for node in nodes {
        res.push_str(&node);
        res.push_str("\n");
    }

    return res;
}