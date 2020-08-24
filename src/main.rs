use futures_util::stream::{Stream, StreamExt, TryStreamExt};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::convert::Infallible;
use std::io;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;

async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("Hello, World".into()))
}

fn error(err: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, err)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // Build TLS configuration.
    let tls_cfg = {
        // Load public certificate.
        let mut cert_reader = io::BufReader::new(std::fs::File::open("./ssl_certs/server.crt")?);
        let certs = rustls::internal::pemfile::certs(&mut cert_reader).unwrap();
        // Load private key.
        let mut key_reader = io::BufReader::new(std::fs::File::open("./ssl_certs/server.key")?);
        // Load and return a single private key.
        let mut keys = rustls::internal::pemfile::pkcs8_private_keys(&mut key_reader).unwrap();
        // Do not use client certificate authentication.
        let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
        // Select a certificate to use.
        cfg.set_single_cert(certs, keys.remove(0)).unwrap();
        // Configure ALPN to accept HTTP/2, HTTP/1.1 in that order.
        cfg.set_protocols(&[b"h2".to_vec(), b"http/1.1".to_vec()]);
        std::sync::Arc::new(cfg)
    };

    // Create a TCP listener via tokio.
    let mut tcp = TcpListener::bind(&addr).await?;
    let tls_acceptor = &TlsAcceptor::from(tls_cfg);
    // Prepare a long-running future stream to accept and serve clients.
    let incoming_tls_stream = tcp
        .incoming()
        .map_err(|e| error(format!("Incoming failed: {:?}", e)))
        // (base: https://github.com/cloudflare/wrangler/pull/1485/files)
        .filter_map(|s| async {
            let client = match s {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("Failed to accept client: {}", e);
                    return None;
                }
            };
            match tls_acceptor.accept(client).await {
                Ok(x) => Some(Ok(x)),
                Err(e) => {
                    eprintln!("Client connection error: {}", e);
                    None
                }
            }
        });

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(hello_world))
    });

    let server = Server::builder(HyperAcceptor {
        acceptor: Box::pin(incoming_tls_stream),
    })
    .serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
    Ok(())
}

struct HyperAcceptor<S> {
    acceptor: core::pin::Pin<Box<S>>,
}

impl<S> hyper::server::accept::Accept for HyperAcceptor<S>
where
    S: Stream<Item = Result<TlsStream<TcpStream>, io::Error>>,
{
    type Conn = TlsStream<TcpStream>;
    type Error = io::Error;

    fn poll_accept(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context,
    ) -> core::task::Poll<Option<Result<Self::Conn, Self::Error>>> {
        self.acceptor.as_mut().poll_next(cx)
    }
}
