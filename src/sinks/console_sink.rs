use crate::sink::{Sink, SinkError};

#[derive(Debug)]
pub struct ConsoleSink;

impl Sink for ConsoleSink {
    fn write(&mut self, data: &Vec<u8>) -> Result<(), SinkError> {
        println!(
            "Writing to console: {}",
            String::from_utf8(data.clone()).expect("Failed to write to the console")
        );
        Ok(())
    }

    fn flush(&mut self) -> Result<(), SinkError> {
        Ok(())
    }
}
