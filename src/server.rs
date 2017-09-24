extern crate hyper;
extern crate futures;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate askama;

use futures::Stream;
use futures::future::Future;
use hyper::server::{Http, Request, Response, Service};
use hyper::header::ContentType;
use hyper::mime;

use askama::Template;

use std::sync::Mutex;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
struct Machine {
    hostname: String,
    global_ip: Option<String>,
    local_ip: Option<String>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    machines: Vec<&'a Machine>,
}

lazy_static! {
    static ref MAP: Mutex<HashMap<String, Machine>> = Mutex::new(HashMap::new());
}

fn handle_ping(request: Request) -> <Ping as Service>::Future {
    let remote_addr = request.remote_addr().map(|h| h.to_string());
    let body = request.body();
    Box::new(body.concat2().and_then(move |chunk| {
        if let Ok(json) = std::str::from_utf8(&*chunk) {
            let j: serde_json::Value = serde_json::from_str(json).expect("malformed json received");
            let obj = j.as_object().expect("json wasn't an object!");

            let hostname = match obj.get("hostname") {
                // TODO(mht): wtf is happnening here?
                Some(hostname) => hostname.as_str().unwrap().to_string(),
                None => {
                    return futures::future::ok(
                        Response::new().with_status(hyper::StatusCode::BadRequest),
                    )
                }
            };
            let machine = Machine {
                hostname: hostname.clone(),
                global_ip: remote_addr,
                local_ip: j["ip"].as_str().map(str::to_string),
            };
            {
                let mut map = MAP.lock().unwrap();
                map.insert(hostname, machine);
            }
        }
        futures::future::ok(Response::new())
    }))
}

fn serve_index(_request: Request) -> <Ping as Service>::Future {
    let data = {
        let map: &HashMap<String, Machine> = &*MAP.lock().unwrap();
        let machines = map.values().collect::<Vec<_>>();
        let template = IndexTemplate { machines };
        template.render().expect("template render failed")
    };
    Box::new(futures::future::ok(
        Response::new()
            .with_header(ContentType(mime::TEXT_HTML))
            .with_body(data),
    ))
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
            _ => Box::new(futures::future::ok(Response::new().with_status(
                hyper::StatusCode::MethodNotAllowed,
            ))),
        }
    }
}

fn main() {
    let addr = "127.0.0.1:8000".parse().unwrap();
    let server = Http::new().bind(&addr, || Ok(Ping)).unwrap();
    server.run().unwrap();
}
