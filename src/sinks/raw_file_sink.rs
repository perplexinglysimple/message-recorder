use std::io::Write;
use std::time::{Duration, Instant};

use getset::Getters;
use log::{error, info};

use crate::sink::{Sink, SinkError};

#[derive(Debug, Getters)]
pub struct RawFileSink {
    #[get = "pub"]
    filename: String,
    writer: std::io::BufWriter<std::fs::File>,
    #[get = "pub"]
    flush_time: Duration,
    #[get = "pub"]
    last_flush: Instant,
}

impl Sink for RawFileSink {
    fn write(&mut self, data: &Vec<u8>) -> Result<(), SinkError> {
        info!(
            "Writing to file {} with {} bytes",
            self.filename,
            data.len()
        );
        self.writer.write_all(data.as_slice())?;
        if self.last_flush.elapsed() >= self.flush_time {
            self.flush()?
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), SinkError> {
        match self.writer.flush() {
            Ok(_) => info!("Flushing writer"),
            Err(e) => {
                error!("Error flushing buffer for {}: {}", self.filename, e);
                return Err(SinkError::IoError(e));
            }
        };
        self.last_flush = Instant::now();
        Ok(())
    }
}

impl RawFileSink {
    pub fn new(filename: String, flush_time_s: i32) -> std::io::Result<Self> {
        let file = std::fs::File::create(&filename)?; // Opens or creates the file
        let writer = std::io::BufWriter::new(file); // Wraps the file in BufWriter
        let flush_time = Duration::new(flush_time_s.try_into().unwrap(), 0);
        let last_flush = Instant::now();
        Ok(RawFileSink {
            filename,
            writer,
            flush_time,
            last_flush,
        })
    }
}
