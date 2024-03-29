extern crate hyper;
extern crate hyper_tls;
extern crate futures;
extern crate tokio_core;
extern crate get_if_addrs;
#[macro_use]
extern crate serde_json;
extern crate hostname;

use futures::Future;
use hyper::{Client, StatusCode, Method, Request};
use hyper::header::{ContentLength, ContentType};
use tokio_core::reactor::Core;

use std::net::{IpAddr, Ipv4Addr};
use std::thread;
use std::time::Duration;

fn get_local_ip() -> Option<Ipv4Addr> {
    get_if_addrs::get_if_addrs()
        .ok()
        .and_then(|addrs| {
            addrs
                .iter()
                .map(|addr| addr.ip())
                .filter(IpAddr::is_ipv4)
                .filter(|a| !a.is_loopback())
                .next()
        })
        .map(|ip| match ip {
            IpAddr::V4(v4) => v4,
            IpAddr::V6(_) => unreachable!(),
        })
}

fn ip_to_json(ip: Ipv4Addr) -> serde_json::Value {
    serde_json::Value::String(format!("{}", ip))
}

fn main() {
    let mut core = Core::new().unwrap();
    let client = Client::configure()
        .connector(hyper_tls::HttpsConnector::new(4, &core.handle()).unwrap())
        .build(&core.handle());

    let target = match std::env::args().nth(1) {
        Some(t) => t,
        None => {
            println!("Usage: ./dyne <target ip:port>");
            return;
        }
    };
    let uri_str = format!("https://{}", target);
    let uri: hyper::Uri = uri_str.parse().expect(
        &format!("Failed to parse uri: {}", uri_str),
    );
    loop {
        let request: Request = {
            let payload = {
                let mut json_obj = json!({});
                if let Some(ip) = get_local_ip() {
                    json_obj["ip"] = ip_to_json(ip);
                }
                json_obj["hostname"] = hostname::get_hostname().unwrap().into();
                json_obj.to_string()
            };
            let mut req = Request::new(Method::Post, uri.clone());
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
        thread::sleep(Duration::from_secs(30));
    }
}
