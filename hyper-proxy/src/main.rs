#![deny(warnings)]

use hyper::{server::conn::http1, service::service_fn, http::HeaderValue};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let in_addr: SocketAddr = ([127, 0, 0, 1], 5555).into();

    let listener = TcpListener::bind(in_addr).await?;

    println!("Listening on http://{}", in_addr);

    loop {
        let (stream, _) = listener.accept().await?;

        let service = service_fn(move |mut req| {
            let uri_string = format!(
                "http://{}{}",
                "me.go.kr",
                req.uri()
                    .path_and_query()
                    .map(|x| x.as_str())
                    .unwrap_or("/")
            );

            let uri = uri_string.parse().unwrap();
            *req.uri_mut() = uri;

            let headers = req.headers_mut();
            headers.insert("proxy-header", HeaderValue::from_static("info"));

            let host = req.uri().host().expect("uri has no host");
            let port = req.uri().port_u16().unwrap_or(80);
            let addr = format!("{}:{}", host, port);
            println!("{:#?}", req);
            async move {
                let client_stream = TcpStream::connect(addr).await.unwrap();

                let (mut sender, conn) =
                    hyper::client::conn::http1::handshake(client_stream).await?;
                    tokio::task::spawn(async move {
                    if let Err(err) = conn.await {
                        println!("Connection failed: {:?}", err);
                    }
                });
                sender.send_request(req).await
            }
        });

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, service)
                .await
            {
                println!("Failed to serve the connection: {:?}", err);
            }
        });
    }
}