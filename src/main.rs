mod process_zmq_connection;
mod sink;
mod sinks;
mod zmq_connection;

use env_logger;
use log::{error, info};
use sinks::console_sink::ConsoleSink;
use sinks::file_sink::FileSink;
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

    // Define multiple ZeroMQ hosts, ports, and topics
    let subscriptions = vec![
        ZmqConnection::new_with_owned(
            "localhost".to_string(),
            "5555".to_string(),
            Some("topic".to_string()),
            "test".to_string(),
        ),
        ZmqConnection::new("localhost", "5557", Some("topic"), "test2"),
        ZmqConnection::new("localhost", "5556", None, "test3"),
    ];

    // Register sinks for each connection
    let subscriptions: Vec<Arc<ZmqConnection>> = subscriptions
        .into_iter()
        .map(|connection| {
            if let Err(e) = connection.register_new_sink(
                "File sink".to_string(),
                Box::new(sink::SinksEnum::FileSink(
                    FileSink::new(connection.get_filename(), MAX_FLUSH_TIME).unwrap(),
                )),
            ) {
                error!("Failed to register File sink: {}", e);
            }
            if let Err(e) = connection.register_new_sink(
                "Console sink".to_string(),
                Box::new(sink::SinksEnum::ConsoleSink(ConsoleSink {})),
            ) {
                error!("Failed to register Console sink: {}", e);
            }
            Arc::new(connection)
        })
        .collect();

    let mut handles = vec![];

    // Spawn a Tokio task for each subscription
    for connection in subscriptions {
        let connection = Arc::clone(&connection);
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
