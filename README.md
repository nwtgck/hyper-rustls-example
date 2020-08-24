# hyper-rustls-example
Simple example of [hyper-rustls](https://github.com/ctz/hyper-rustls)

## Run

Run as follows.

```bash
cargo run
```

Access to the server as follows.

```bash
curl -k https://localhost:3000/
```

## Make self-signed certificates by yourself

You can make certificates as follows.

```bash
mkdir ssl_certs && cd ssl_certs && openssl req -x509 -newkey rsa:4096 -keyout _server.key -out server.crt -days 365 -sha256 -nodes --subj '/CN=localhost/' && openssl rsa -in _server.key -out server.key && rm _server.key && cd -
```
(ref: [Can't run example with system certificate · Issue #26 · ctz/rustls](https://github.com/ctz/rustls/issues/26#issuecomment-565515486))
