use crate::sinks::console_sink::ConsoleSink;
use crate::sinks::file_sink::FileSink;

use log::debug;

#[derive(Debug)]
pub enum SinkError {
    IoError(std::io::Error),
}

impl From<std::io::Error> for SinkError {
    fn from(err: std::io::Error) -> SinkError {
        SinkError::IoError(err)
    }
}

impl std::fmt::Display for SinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SinkError::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

pub trait Sink: Sync + Send {
    fn write(&mut self, data: &Vec<u8>) -> Result<(), SinkError>;
}

#[derive(Debug)]
pub struct MockSink {
    pub data: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
}

impl Sink for MockSink {
    fn write(&mut self, data: &Vec<u8>) -> Result<(), SinkError> {
        let mut stored_data = self.data.lock().unwrap();
        debug!("Before {:?}", stored_data);
        stored_data.clone_from(&data);
        debug!("After {:?}", stored_data);
        Ok(())
    }
}

#[derive(Debug)]
pub enum SinksEnum {
    ConsoleSink(ConsoleSink),
    FileSink(FileSink),
    MockSink(MockSink),
}

impl SinksEnum {
    fn write(&mut self, data: &Vec<u8>) -> Result<(), SinkError> {
        match self {
            SinksEnum::MockSink(sink) => sink.write(data),
            SinksEnum::ConsoleSink(sink) => sink.write(data),
            SinksEnum::FileSink(sink) => sink.write(data),
        }
    }
}
