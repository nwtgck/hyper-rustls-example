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
mkdir ssl_certs && cd ssl_certs && openssl req -x509 -newkey rsa:4096 -keyout server.key -out server.crt -days 365 -sha256 -nodes --subj '/CN=localhost/' && cd -
```

