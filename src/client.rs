extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate get_if_addrs;
#[macro_use]
extern crate json;
extern crate hostname;

use futures::Future;
use hyper::{Client, StatusCode, Method, Request};
use hyper::header::{ContentLength, ContentType};
use tokio_core::reactor::Core;

use std::net::{IpAddr, Ipv4Addr};

fn get_local_ip() -> Option<Ipv4Addr> {
    get_if_addrs::get_if_addrs().ok().and_then(|addrs| {
        addrs
            .iter()
            .map(|addr| addr.ip())
            .filter(IpAddr::is_ipv4)
            .filter(|a| !a.is_loopback())
            .next()
    }).map(|ip| match ip {
        IpAddr::V4(v4) => v4,
        IpAddr::V6(_) => unreachable!()
    })
}

fn ip_to_json(ip: Ipv4Addr) -> json::JsonValue {
    json::JsonValue::String(format!("{}", ip))
}

fn main() {
    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());

    let uri = "http://127.0.0.1:8000".parse().unwrap();
    let request: Request = {
        let payload = {
            let mut json_obj = object!();
            if let Some(ip) = get_local_ip() {
                json_obj["ip"] = ip_to_json(ip);
            }
            json_obj["hostname"] = hostname::get_hostname().unwrap().into();
            json::stringify(json_obj)
        };
        let mut req = Request::new(Method::Post, uri);
        req.headers_mut().set(ContentType::json());
        req.headers_mut().set(ContentLength(payload.len() as u64));
        req.set_body(payload);
        req
    };

    let work = client.request(request).map(|res| match res.status() {
        StatusCode::Ok => {}
        status => {
            println!("Got other status code back: {:?}", status);
        }
    });
    core.run(work).unwrap();
}
