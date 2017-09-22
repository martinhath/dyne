extern crate hyper;
extern crate futures;
extern crate json;

use futures::Stream;
use futures::future::Future;
use hyper::server::{Http, Request, Response, Service};

struct Ping;

impl Service for Ping {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;

    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, _req: Request) -> Self::Future {
        println!("{:#?}", _req);
        let remote_addr = _req.remote_addr().map(|h| h.to_string());
        let body = _req.body();
        Box::new(body.concat2().and_then(move |chunk| {
            if let Ok(json) = std::str::from_utf8(&*chunk) {
                let j = json::parse(json).expect("malformed json received");
                if let Some(ip) = remote_addr {
                    println!("global ip: {:?}", ip);
                }
                if let Some(ip) = j["ip"].as_str() {
                    println!("local ip: {:?}", ip);
                }
                if let Some(hostname) = j["hostname"].as_str() {
                    println!("hostname: {:?}", hostname);
                }
            }
            futures::future::ok(Response::new())
        }))
    }
}

fn main() {
    let addr = "127.0.0.1:8000".parse().unwrap();
    let server = Http::new().bind(&addr, || Ok(Ping)).unwrap();
    server.run().unwrap();
}
