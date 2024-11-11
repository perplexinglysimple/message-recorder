use crate::zmq_connection::{MessageRecorderError, ZmqConnection};
use futures::TryStreamExt;
use log::{debug, error, info};
use prost::Message;
use std::sync::Arc;
use tmq::{subscribe, Context};

pub async fn process_zmq_connection(
    connection: &Arc<ZmqConnection>,
) -> Result<(), MessageRecorderError> {
    // Build the connection string
    let host = connection.get_host();
    let socket;

    // Set up topic subscription
    let topic = match connection.get_topic() {
        None => {
            info!("Subscribed to NO TOPIC on {}", host);
            socket = match subscribe(&Context::new()).connect(&host) {
                Ok(res) => Ok(res),
                Err(err) => Err(MessageRecorderError::TmqError(err)),
            }?;
            ""
        }
        Some(topic) => {
            info!("Subscribed to topic {} on {}", topic, host);
            socket = match subscribe(&Context::new()).connect(&host) {
                Ok(res) => Ok(res),
                Err(err) => Err(MessageRecorderError::TmqError(err)),
            }?;
            topic
        }
    };
    let mut subscribe = match socket.subscribe(topic.as_bytes()) {
        Ok(res) => Ok(res),
        Err(err) => Err(MessageRecorderError::TmqError(err)),
    }?;

    loop {
        match subscribe.try_next().await {
            Ok(possible_message) => {
                debug!("Recieved {:?}", possible_message);
                match possible_message {
                    Some(message) => {
                        // tmq messages are Vec<Vec<u8>>, where the first frame is the topic and others are message parts
                        if let Some(received_topic) = message
                            .0
                            .get(0)
                            .and_then(|frame| std::str::from_utf8(frame).ok())
                        {
                            // Check topic matching if a topic is specified
                            if let Some(expected_topic) = connection.get_topic() {
                                if received_topic != expected_topic {
                                    info!(
                                        "Received unexpected topic: {}. Expected: {}",
                                        received_topic, expected_topic
                                    );
                                    continue; // Skip if topic doesn't match
                                }
                            }
                        }

                        // Collect message data (excluding the first frame if it's the topic)
                        let data: Vec<u8> = match connection.get_topic() {
                            Some(_) => message
                                .iter()
                                .skip(1)
                                .flat_map(|frame| frame.iter())
                                .copied()
                                .collect(),
                            None => message
                                .iter()
                                .flat_map(|frame| frame.iter())
                                .copied()
                                .collect(),
                        };

                        // Pass data to connection sinks
                        if let Err(e) = connection.use_sinks(&data) {
                            error!("Failed to use sinks with error {} from {}", e, &connection);
                        }
                    }
                    None => error!("Failed to recieve any data from {}", &connection),
                }
            }
            Err(e) => error!("Error receiving message: {} from {}", e, &connection),
        }
    }
}
