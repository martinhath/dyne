extern crate hyper;
extern crate futures;
extern crate tokio_core;

use std::io::{self, Write};
use futures::{Future, Stream};
use hyper::Client;
use tokio_core::reactor::Core;

fn main() {
    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());

    let uri = "http://127.0.0.1:8000".parse().unwrap();
    let work = client.get(uri).and_then(|res| {
        println!("response: {}", res.status());
        res.body().for_each(|chunk| {
            io::stdout()
                .write_all(&chunk)
                .map_err(From::from)
        })
    });
    core.run(work).unwrap();
}
