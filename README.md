# Dyne - A dynamic IP recorder

This utility allows you to keep track of the IP address of your devices.
We include a client and a server: `dyne` and `dyned`.
The client `POST`s to the server occationally, and upon `GET`, the server returns a `.html` page, showing
all devices recorded. No information is saved on restarts.

If `dyned` is ran behind another webserver, eg. Nginx, we depend on the `X-Forwarded-For` header to get the global IP address.

## Usage

Run the server with

```bash
cargo run --release --bin dyned
```

and the client with

```bash
cargo run --release --bin dyne <ip of server>
```
