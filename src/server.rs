extern crate notify;
extern crate pathdiff;
extern crate iron;
extern crate router;
extern crate url;

use std::path::{PathBuf};
use std::sync::{Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read,Write};
use std::ops::Deref;
use std::net::IpAddr;

use url::form_urlencoded;

use iron::prelude::*;
use iron::status;
use router::Router;


pub fn init_server(router: &mut Router, file_timestamps: &'static Mutex<HashMap<PathBuf,Duration>>, other_nodes: &'static Mutex<Vec<String>>) {

    router.get("/", |_req: &mut Request| -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Hello world!")))
    }, "test");

    // Pull file <-
    router.get("/file/:file_path", |req: &mut Request| -> IronResult<Response> {

        // get file path
        let file_path = req.extensions.get::<Router>().unwrap().find("file_path").unwrap_or("/");
        let parse: Vec<(String, String)> = form_urlencoded::parse(file_path.as_bytes()).into_owned().collect();
        let file_path = parse[0].0.to_owned();

        // get file contents
        let mut file = File::open(file_path).unwrap();
        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        
        return Ok(Response::with((status::Ok, buf)));
    }, "get_file");

    // Push file change ->
    router.post("/file/:file_path", move |req: &mut Request| -> IronResult<Response> {

        // get file path
        let file_path = req.extensions.get::<Router>().unwrap().find("file_path").unwrap();
        let parse: Vec<(String, String)> = form_urlencoded::parse(file_path.as_bytes()).into_owned().collect();
        let file_path = parse[0].0.to_owned();

        // get request body
        let mut buf: Vec<u8> = Vec::new();
        req.body.read_to_end(&mut buf).unwrap();

        // write to file
        let mut file = File::open(file_path).unwrap();
        file.write(&buf).unwrap();
        
        // record new timestamp
        let mut locked = file_timestamps.lock().unwrap();
        locked.insert(PathBuf::new(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap());
        
        return Ok(Response::with(status::Ok));
    }, "post_file");


    router.post("/greet/:remote_port", move |req: &mut Request| -> IronResult<Response> {
        
        // get remote address and port
        let remote_port = req.extensions.get::<Router>().unwrap().find("remote_port").unwrap();
        let other_addr = match req.remote_addr.ip() {
            IpAddr::V4(addr) => addr.to_string(),
            IpAddr::V6(addr) => String::from("[") + &addr.to_string() + "]"
        } + ":" + remote_port;

        print!("Greeted by {}\n", &other_addr);

        // check if we already know this node
        let mut locked = other_nodes.lock().unwrap();
        let locked_ref = Deref::deref(&locked);
        let mut found = false;
        for known in locked_ref {
            if known == &other_addr { found = true };
        }
        if !found {
            locked.push(other_addr);
        }

        return Ok(Response::with(status::Ok));
    }, "greet");

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
