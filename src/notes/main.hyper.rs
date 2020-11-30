use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

mod chord;
mod swim;

struct Handle {}

impl Handle {
    pub async fn incoming(self: Self, mut req: Request<Body>) -> Result<Response<Body>,
        Infallible> {
        Ok::<_, Infallible>(Response::<Body>::new("Hello world!".into()))
    }
}

// async fn handle(_: Request<Body>) -> Result<Response<Body>, Infallible> {
//     Ok(Response::new("Hello, World!".into()))
// }

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let handle = Arc::new(Handle {});

    // make_service_fn running on each connection
    let make_svc = make_service_fn(move |_conn| {
        let handle = Arc::clone(&handle);
        async {
            // service_fn running on each request
            Ok::<_, Infallible>(service_fn(move |req| {
                // handle.incoming(req)
                let rp = handle.clone();
                async move {
                    rp.incoming(req).await
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}