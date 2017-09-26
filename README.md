# Dyne - A dynamic IP recorder

This utility allows you to keep track of the IP address of your devices.
We include a client and a server: `dyne` and `dyned`.

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
