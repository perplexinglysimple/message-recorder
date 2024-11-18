use crate::sinks::compressed_file_sink::CompressedFileSink;
use crate::sinks::console_sink::ConsoleSink;
use crate::sinks::file_sink::FileSink;
use crate::sinks::message_counter::MessageCounter;

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
    fn flush(&mut self) -> Result<(), SinkError>;
}

#[derive(Debug)]
pub enum SinksEnum {
    ConsoleSink(ConsoleSink),
    FileSink(FileSink),
    CompressedFileSink(CompressedFileSink),
    MessageCounter(MessageCounter),
}
