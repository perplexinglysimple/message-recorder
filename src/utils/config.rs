use crate::sink::SinksEnum;
use crate::sinks::compressed_file_sink::CompressedFileSink;
use crate::sinks::console_sink::ConsoleSink;
use crate::sinks::message_counter::MessageCounter;
use crate::sinks::prometheus_sink::PrometheusSink;
use crate::{sinks::file_sink::FileSink, zmq_connection::ZmqConnection};

use log::{error, trace, warn};
use prometheus_client::registry::Registry;
use serde::Deserialize;

use figment::{
    providers::{Format, Yaml},
    Figment,
};

#[derive(Debug, Deserialize)]
struct Sink {
    sink_type: String,
    name: Option<String>,
    flush_time: Option<i32>,
    compression_level: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct Connections {
    addr: String,
    port: i32,
    topic: Option<String>,
    file_extension: String,
    sinks: Option<Vec<Sink>>,
}

#[derive(Debug, Deserialize)]
struct Config {
    connections: Vec<Connections>,
}

fn create_filepath(filename: String, out_dir: &Option<String>) -> String {
    match out_dir {
        Some(val) => std::path::Path::new(val)
            .join(filename)
            .to_str()
            .expect("Failed to create a filepath to write")
            .to_string(),
        None => filename,
    }
}

pub fn read_config(
    filename: &str,
    out_dir: Option<String>,
    registry: &mut Registry,
) -> Vec<ZmqConnection> {
    trace!(
        "Config file contains this data:\n{}",
        std::fs::read_to_string(filename).expect("Could not open the config file!")
    );
    let config: Config = Figment::new()
        .merge(Yaml::file(filename))
        .extract()
        .unwrap();
    let mut connections = Vec::new();

    for conn_cfg in config.connections {
        let zmq_conn = ZmqConnection::new_with_owned(
            conn_cfg.addr.clone(),
            conn_cfg.port.to_string(),
            conn_cfg.topic,
            conn_cfg.file_extension,
        );
        if let Some(sinks_cfg) = conn_cfg.sinks {
            for sink_cfg in sinks_cfg {
                let sink_enum = match sink_cfg.sink_type.as_str() {
                    "File Sink" => SinksEnum::FileSink(
                        FileSink::new(
                            create_filepath(zmq_conn.get_filename(), &out_dir),
                            sink_cfg.flush_time.unwrap_or(0),
                        )
                        .unwrap(),
                    ),
                    "Console Sink" => SinksEnum::ConsoleSink(ConsoleSink {}),
                    "Compressed Sink" => SinksEnum::CompressedFileSink(
                        CompressedFileSink::new(
                            create_filepath(zmq_conn.get_filename(), &out_dir),
                            sink_cfg.flush_time.unwrap_or(0),
                            sink_cfg.compression_level.unwrap_or(1),
                        )
                        .unwrap(),
                    ),
                    "Message Counter" => SinksEnum::MessageCounter(MessageCounter::new()),
                    "Prometheus Client" => SinksEnum::PrometheusSink(PrometheusSink::new(
                        registry,
                        sink_cfg
                            .name
                            .unwrap_or(format!("{}.{}", conn_cfg.addr, conn_cfg.port))
                            .as_str(),
                    )),
                    _ => {
                        warn!("There is no sink for '{}'", sink_cfg.sink_type.as_str());
                        continue;
                    }
                };
                let sink_repr = format!("{:?}", sink_enum);
                if zmq_conn.register_new_sink(sink_enum).is_err() {
                    error!("Failed to register sink {}", sink_repr);
                }
            }
        }

        connections.push(zmq_conn);
    }
    trace!("Built the connection vector:\n{:?}", connections);

    connections
}
