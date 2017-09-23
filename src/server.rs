extern crate hyper;
extern crate futures;
extern crate json;
#[macro_use]
extern crate lazy_static;

use futures::Stream;
use futures::future::Future;
use hyper::server::{Http, Request, Response, Service};

use std::sync::Mutex;
use std::collections::HashMap;

#[derive(Debug)]
struct Machine {
    hostname: String,
    global_ip: Option<String>,
    local_ip: Option<String>,
}

lazy_static! {
    static ref MAP: Mutex<HashMap<String, Machine>> = Mutex::new(HashMap::new());
}

fn handle_ping(request: Request) -> <Ping as Service>::Future {
    let remote_addr = request.remote_addr().map(|h| h.to_string());
    let body = request.body();
    Box::new(body.concat2().and_then(move |chunk| {
        if let Ok(json) = std::str::from_utf8(&*chunk) {
            let j = json::parse(json).expect("malformed json received");
            let hostname = match j["hostname"].as_str() {
                Some(hostname) => hostname,
                None => {
                    return futures::future::ok(
                        Response::new().with_status(hyper::StatusCode::BadRequest),
                    )
                }
            };
            let machine = Machine {
                hostname: hostname.to_string(),
                global_ip: remote_addr,
                local_ip: j["ip"].as_str().map(str::to_string),
            };
            {
                let mut map = MAP.lock().unwrap();
                map.insert(hostname.to_string(), machine);
            }
        }
        futures::future::ok(Response::new())
    }))
}

fn serve_index(request: Request) -> <Ping as Service>::Future {
    // TODO(mht): serve index.html, with `MAP` data.
    Box::new(futures::future::ok(Response::new()))
}

struct Ping;
impl Service for Ping {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;

    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        match req.method() {
            &hyper::Method::Post => handle_ping(req),
            &hyper::Method::Get => serve_index(req),
            _ => Box::new(futures::future::ok(Response::new().with_status(hyper::StatusCode::MethodNotAllowed)))
        }
    }
}

fn main() {
    let addr = "127.0.0.1:8000".parse().unwrap();
    let server = Http::new().bind(&addr, || Ok(Ping)).unwrap();
    server.run().unwrap();
}
