use crate::sinks::compressed_file_sink::CompressedFileSink;
use crate::sinks::console_sink::ConsoleSink;
use crate::{sinks::file_sink::FileSink, zmq_connection::ZmqConnection};
use crate::sink::SinksEnum;

use serde::Deserialize;

use figment::{Figment, providers::{Format, Yaml}};

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

pub fn read_config(filename: &str) -> Vec<ZmqConnection> {
    let config: Config = Figment::new().merge(Yaml::file(filename)).extract().unwrap();
    let mut connections = Vec::new();

    for conn_cfg in config.connections {
        let mut zmq_conn = ZmqConnection::new_with_owned(
            conn_cfg.addr,
            conn_cfg.port.to_string(),
            conn_cfg.topic,
            conn_cfg.file_extension,
        );
        if let Some(sinks_cfg) = conn_cfg.sinks {
            for sink_cfg in sinks_cfg {
                let sink_enum = match sink_cfg.sink_type.as_str() {
                    "File Sink" => SinksEnum::FileSink(FileSink::new(zmq_conn.get_filename(), sink_cfg.flush_time.unwrap_or(0)).unwrap()),
                    "Console Sink" => SinksEnum::ConsoleSink(ConsoleSink{}),
                    "Compressed Sink" => SinksEnum::CompressedFileSink(CompressedFileSink::new(zmq_conn.get_filename(), sink_cfg.flush_time.unwrap_or(0), sink_cfg.compression_level.unwrap_or(1))),
                    _ => continue,
                };
                let sink_name = sink_cfg.name.unwrap_or(sink_cfg.sink_type);
                zmq_conn.register_new_sink(sink_name, Box::new(sink_enum));
            }
        }

        connections.push(zmq_conn);
    }

    connections
}