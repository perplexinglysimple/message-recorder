mod message_decoding;
mod process_zmq_connection;
mod sink;
mod sinks;
mod utils;
mod zmq_connection;

use clap::Parser;
use env_logger;
use log::{error, info};
use prometheus_client::registry::{self, Registry};
use prometheus_client::encoding::text::encode;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::io;
use futures::future::BoxFuture;
use tokio::{
    net::TcpListener,
    pin,
    signal::unix::{signal, SignalKind},
};
use hyper::{
    body::{Bytes, Incoming},
    server::conn::http1,
    service::service_fn,
    Request, Response,
};
use hyper_util::rt::TokioIo;
use http_body_util::{combinators, BodyExt, Full};
use crate::process_zmq_connection::process_zmq_connection;


pub async fn start_metrics_server(metrics_addr: SocketAddr, registry: Registry) {
    eprintln!("Starting metrics server on {metrics_addr}");

    let registry = Arc::new(registry);

    let tcp_listener = TcpListener::bind(metrics_addr).await.unwrap();
    let server = http1::Builder::new();
    while let Ok((stream, _)) = tcp_listener.accept().await {
        let mut shutdown_stream = signal(SignalKind::terminate()).unwrap();
        let io = TokioIo::new(stream);
        let server_clone = server.clone();
        let registry_clone = registry.clone();
        tokio::task::spawn(async move {
            let conn = server_clone.serve_connection(io, service_fn(make_handler(registry_clone)));
            pin!(conn);
            tokio::select! {
                _ = conn.as_mut() => {}
                _ = shutdown_stream.recv() => {
                    conn.as_mut().graceful_shutdown();
                }
            }
        });
    }
}

pub fn make_handler(
    registry: Arc<Registry>,
) -> impl Fn(Request<Incoming>) -> BoxFuture<'static, io::Result<Response<combinators::BoxBody<Bytes, hyper::Error>>>> {
    // This closure accepts a request and responds with the OpenMetrics encoding of our metrics.
    move |_req: Request<Incoming>| {
        let reg = registry.clone();

        Box::pin(async move {
            let mut buf = String::new();
            encode(&mut buf, &reg.clone())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                .map(|_| {
                    let body = full(Bytes::from(buf));
                    Response::builder()
                        .header(
                            hyper::header::CONTENT_TYPE,
                            "application/openmetrics-text; version=1.0.0; charset=utf-8",
                        )
                        .body(body)
                        .unwrap()
                })
        })
    }
}

pub fn full(body: Bytes) -> combinators::BoxBody<Bytes, hyper::Error> {
    Full::new(body).map_err(|never| match never {}).boxed()
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = crate::utils::arg_parser::Args::parse();
    let log_level = match args.log_level.to_lowercase().as_str() {
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "error" => log::LevelFilter::Error,
        "trace" => log::LevelFilter::Trace,
        "warn" => log::LevelFilter::Warn,
        "err" => log::LevelFilter::Error,
        _ => panic!("Could not determine the log level!"),
    };
    println!("Configured the log level: {}", log_level);
    env_logger::builder().filter_level(log_level).init();
    info!("Starting up");
    info!(
        "Reading the file {} for the record config",
        &args.config_loc
    );
    info!("Found outdir of {:?}", args.out_dir);
    let mut registry = Registry::default();

    let subscriptions = utils::config::read_config(&args.config_loc, args.out_dir, &mut registry);

    let mut handles = vec![];

    // Spawn a Tokio task for each subscription
    for connection in subscriptions {
        info!("Subscribing to connection: {:?}", connection);

        let handle = tokio::spawn(async move {
            match process_zmq_connection(&connection).await {
                Ok(value) => error!(
                    "Wait exited for connection {:?} without an error {:?}",
                    connection, value
                ),
                Err(e) => error!(
                    "Error received from subscribe function for connection {:?}: {}",
                    connection, e
                ),
            };
        });
        handles.push(handle);
    }

    handles.push(tokio::spawn(start_metrics_server(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8001), registry)));

    for handle in handles {
        if let Err(e) = handle.await {
            error!("A task failed with error: {:?}", e);
        }
    }
}
