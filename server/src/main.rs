#[macro_use]
extern crate log;

use std::{convert::Infallible, fs, process::exit};

use assets::include_resources;
use config::Config;
use http_body_util::Full;
use hyper::{
    body::{Body, Bytes},
    server::conn::http1,
    service::service_fn,
    Request, Response,
};
use hyper_util::rt::{TokioIo, TokioTimer};
use maud::DOCTYPE;
use tokio::{net::TcpListener, spawn};

mod config;

async fn service(req: Request<impl Body>) -> Result<Response<Full<Bytes>>, Infallible> {
    include!(concat!(env!("OUT_DIR"), "/client_files.rs"));

    include_resources!(
        x.png => if req.uri().path() == format!("/assets{}", x.path).as_str() {
            return Ok(Response::builder()
                .header("Content-Type", x.mime)
                .body(Full::new(Bytes::from_static(x.bytes))).unwrap());
        },
        x.ttf => if req.uri().path() == format!("/assets{}", x.path).as_str() {
            return Ok(Response::builder()
                .header("Content-Type", x.mime)
                .body(Full::new(Bytes::from_static(x.bytes))).unwrap());
        },
    );

    Ok(Response::builder()
        .status(404)
        .header("Content-Type", "text/html")
        .body(Full::new(Bytes::from(
            maud::html! {
                (DOCTYPE)
                html {
                    head {
                        meta charset="UTF-8";
                        title { "404 | Not Found" }
                    }
                    body {
                        h1 { "Not Found" }
                        p { "This url could not be found on this server" }
                    }
                }
            }
            .0,
        )))
        .unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    pretty_env_logger::init();

    let config: Config = {
        let mut iter = std::env::args();
        let exe = iter.next().unwrap_or_default();
        let file = iter.next().unwrap_or_else(|| {
            eprintln!("usage: {exe} <config>");
            exit(1);
        });
        toml::from_str(&fs::read_to_string(file)?)?
    };

    let mut handles = vec![];

    for x in config.http {
        let listener = TcpListener::bind(x.bind).await?;
        let join = spawn(async move {
            info!("Listening on (http) {}", x.bind);
            loop {
                let (tcp, _) = match listener.accept().await {
                    Ok(x) => x,
                    Err(why) => {
                        error!("Listener failure: {why}");
                        break;
                    }
                };
                let io = TokioIo::new(tcp);
                spawn(async move {
                    if let Err(why) = http1::Builder::new()
                        .timer(TokioTimer::new())
                        .serve_connection(io, service_fn(service))
                        .await
                    {
                        error!("Accept failed: {why}");
                    }
                });
            }
        });
        handles.push(join);
    }

    for x in handles {
        x.await?;
    }

    Ok(())
}
