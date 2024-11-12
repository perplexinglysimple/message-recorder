use log::info;

use crate::sink::{Sink, SinkError};
use std::borrow::BorrowMut;

use crate::sink::SinksEnum;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum MessageRecorderError {
    TmqError(tmq::TmqError),
    IoError(std::io::Error),
    PoisonError(String),
    SinkError(SinkError),
}

impl From<tmq::TmqError> for MessageRecorderError {
    fn from(err: tmq::TmqError) -> MessageRecorderError {
        MessageRecorderError::TmqError(err)
    }
}

impl From<std::io::Error> for MessageRecorderError {
    fn from(err: std::io::Error) -> MessageRecorderError {
        MessageRecorderError::IoError(err)
    }
}

impl From<SinkError> for MessageRecorderError {
    fn from(err: SinkError) -> MessageRecorderError {
        MessageRecorderError::SinkError(err)
    }
}

impl From<String> for MessageRecorderError {
    fn from(err: String) -> MessageRecorderError {
        MessageRecorderError::PoisonError(err)
    }
}

impl std::fmt::Display for MessageRecorderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MessageRecorderError::IoError(err) => write!(f, "IO error: {:?}", err),
            MessageRecorderError::TmqError(err) => write!(f, "Zmq error: {:?}", err),
            MessageRecorderError::PoisonError(err) => write!(f, "Poison error: {:?}", err),
            MessageRecorderError::SinkError(err) => write!(f, "Sink error: {:?}", err),
        }
    }
}

#[derive(Debug)]
pub struct ZmqConnection {
    addr: String,
    port: String,
    topic: Option<String>,
    file_extension: String,
    sinks: Arc<Mutex<HashMap<String, Box<SinksEnum>>>>,
}

impl ZmqConnection {
    pub fn new(addr: &str, port: &str, topic: Option<&str>, file_extension: &str) -> Self {
        Self {
            addr: addr.to_string(),
            port: port.to_string(),
            topic: topic.map(|t| t.to_string()),
            file_extension: file_extension.to_string(),
            sinks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn new_with_owned(
        addr: String,
        port: String,
        topic: Option<String>,
        file_extension: String,
    ) -> Self {
        Self {
            addr,
            port,
            topic,
            file_extension,
            sinks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_host(&self) -> String {
        format!("tcp://{}:{}", &self.addr, &self.port)
    }

    pub fn get_topic(&self) -> &Option<String> {
        &self.topic
    }

    pub fn get_file_extension(&self) -> &String {
        &self.file_extension
    }

    pub fn get_filename(&self) -> String {
        match self.get_topic() {
            None => format!("{}_NO_TOPIC.{}", self.get_host(), self.get_file_extension()),
            Some(topic_str) => format!(
                "{}_{}.{}",
                self.get_host(),
                topic_str,
                self.get_file_extension()
            ),
        }
        .replace(":", "_")
        .replace("/", "_")
        .replace("\\", "_")
    }

    pub fn register_new_sink(
        &self,
        sink_name: String,
        new_sink: Box<SinksEnum>,
    ) -> Result<(), MessageRecorderError> {
        match self.sinks.lock() {
            Ok(mut res) => {
                res.insert(sink_name, new_sink);
                Ok(())
            }
            Err(e) => Err(MessageRecorderError::PoisonError(format!(
                "Failed to get lock in register new sink: {}",
                e
            ))),
        }
    }

    pub fn use_sinks(&self, data: &Vec<u8>) -> Result<(), MessageRecorderError> {
        match self.sinks.lock() {
            Ok(mut res) => {
                for (sink_name, sink) in res.iter_mut() {
                    info!("Logging to {} with size {}", sink_name, data.len());
                    match sink.borrow_mut() {
                        SinksEnum::ConsoleSink(cs) => cs.write(&data)?,
                        SinksEnum::FileSink(fs) => fs.write(&data)?,
                        SinksEnum::MockSink(ms) => ms.write(&data)?,
                        SinksEnum::CompressedFileSink(cs) => cs.write(data)?,
                    };
                }
                Ok(())
            }
            Err(e) => Err(MessageRecorderError::PoisonError(format!(
                "Failed to lock {}",
                e
            ))),
        }
    }
}

impl std::fmt::Display for ZmqConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let sink_number = match self.sinks.lock() {
            Ok(res) => res.keys().len().to_string(),
            Err(_) => "Could not lock sink lock".to_string(),
        };
        write!(
            f,
            "ZmqConnection: {{ addr:{}, port:{}, topic:{:?}, file_extension:{}, len(sinks):{} }}",
            self.addr, self.port, self.topic, self.file_extension, sink_number
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_new() {
        let connection = ZmqConnection::new("127.0.0.1", "5555", Some("test_topic"), "test");
        assert_eq!(connection.addr, "127.0.0.1");
        assert_eq!(connection.port, "5555");
        assert_eq!(connection.topic, Some("test_topic".to_string()));
    }

    #[test]
    fn test_new_with_owned() {
        let connection = ZmqConnection::new_with_owned(
            "192.168.1.1".to_string(),
            "6666".to_string(),
            Some("another_topic".to_string()),
            "test".to_string(),
        );
        assert_eq!(connection.addr, "192.168.1.1");
        assert_eq!(connection.port, "6666");
        assert_eq!(connection.topic, Some("another_topic".to_string()));
    }

    #[test]
    fn test_get_host() {
        let connection = ZmqConnection::new("localhost", "5555", None, "test");
        assert_eq!(connection.get_host(), "tcp://localhost:5555");
    }

    #[test]
    fn test_get_topic_some() {
        let connection = ZmqConnection::new("127.0.0.1", "5555", Some("test_topic"), "test");
        assert_eq!(connection.get_topic(), &Some("test_topic".to_string()));
    }

    #[test]
    fn test_get_topic_none() {
        let connection = ZmqConnection::new("127.0.0.1", "5555", None, "test");
        assert_eq!(connection.get_topic(), &None);
    }

    #[test]
    fn test_get_file_extension() {
        let connection = ZmqConnection::new("127.0.0.1", "5555", None, "test_ext");
        assert_eq!(connection.get_file_extension(), "test_ext");
    }

    #[test]
    fn test_get_filename_with_topic() {
        let connection = ZmqConnection::new("127.0.0.1", "5555", Some("test_topic"), "txt");
        let expected_filename = "tcp___127.0.0.1_5555_test_topic.txt";
        assert_eq!(connection.get_filename(), expected_filename);
    }

    #[test]
    fn test_get_filename_without_topic() {
        let connection = ZmqConnection::new("127.0.0.1", "5555", None, "log");
        let expected_filename = "tcp___127.0.0.1_5555_NO_TOPIC.log";
        assert_eq!(connection.get_filename(), expected_filename);
    }

    #[test]
    fn test_register_new_sink() {
        let connection = ZmqConnection::new("127.0.0.1", "5555", None, "test");
        let data_storage = Arc::new(Mutex::new(Vec::new()));
        let mock_sink = sink::MockSink {
            data: data_storage.clone(),
        };
        let sink_enum = SinksEnum::MockSink(mock_sink);

        connection
            .register_new_sink("mock_sink".to_string(), Box::new(sink_enum))
            .expect("Failed to register new sink");

        // Check that the sink has been registered
        let sinks = connection.sinks.lock().unwrap();
        assert!(sinks.contains_key("mock_sink"));
    }

    #[test]
    fn test_use_sinks() {
        let connection = ZmqConnection::new("127.0.0.1", "5555", None, "test");
        let mock_sink = sink::MockSink {
            data: Arc::new(Mutex::new(Vec::new())),
        };
        let sink_enum = SinksEnum::MockSink(mock_sink);

        connection
            .register_new_sink("mock_sink".to_string(), Box::new(sink_enum))
            .expect("Failed to register new sink");

        let test_data = b"Hello, World!".to_vec();
        connection
            .use_sinks(&test_data)
            .expect("Failed to use sinks");

        let sinks = connection.sinks.lock().unwrap();
        let stored_data = sinks.get("mock_sink").unwrap();

        match &**stored_data {
            SinksEnum::MockSink(to_test) => {
                let data_guard = to_test.data.lock().unwrap();
                let data = &*data_guard;
                assert_eq!(data, &test_data);
            }
            _ => panic!("Expected MockSink"),
        }
    }
}
