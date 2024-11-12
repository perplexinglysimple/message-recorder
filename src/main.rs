mod process_zmq_connection;
mod sink;
mod sinks;
mod zmq_connection;
mod config;

use env_logger;
use log::{error, info};
use sinks::console_sink::ConsoleSink;
use sinks::file_sink::FileSink;
use sinks::compressed_file_sink::CompressedFileSink;
use std::sync::Arc;

use crate::process_zmq_connection::process_zmq_connection;
use crate::zmq_connection::ZmqConnection;

const MAX_FLUSH_TIME: i32 = 5;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Initialize the logger
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    info!("Starting up");

    let subscriptions = config::read_config("config/config.yml");

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

    for handle in handles {
        if let Err(e) = handle.await {
            error!("A task failed with error: {:?}", e);
        }
    }
}
