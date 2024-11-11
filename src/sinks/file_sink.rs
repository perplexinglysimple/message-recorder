use std::io::Write;
use std::time::{Duration, Instant};

use byteorder::{BigEndian, WriteBytesExt};
use log::{debug, error, info};

use crate::sink::{Sink, SinkError};

#[derive(Debug)]
pub struct FileSink {
    filename: String,
    writer: std::io::BufWriter<std::fs::File>,
    flush_time: Duration,
    last_flush: Instant,
}

impl Sink for FileSink {
    fn write(&mut self, data: &Vec<u8>) -> Result<(), SinkError> {
        info!(
            "Writing to file {}: {}",
            self.filename,
            String::from_utf8(data.clone()).expect("Failed to write to the file")
        );
        let mut data_size_vec = vec![];
        WriteBytesExt::write_u64::<BigEndian>(&mut data_size_vec, data.len().try_into().unwrap())?;
        match self.writer.write_all(&data_size_vec) {
            Ok(_) => info!(
                "Wrote down {} bytes to {}",
                data_size_vec.len(),
                self.filename
            ),
            Err(e) => {
                error!(
                    "Error writing buffer for {}: {} with size {}",
                    self.filename,
                    e,
                    data_size_vec.len()
                );
                return Err(SinkError::IoError(e));
            }
        };
        self.writer.write_all(data.as_slice())?;
        if self.last_flush.elapsed() >= self.flush_time {
            match self.writer.flush() {
                Ok(_) => info!("Flushing writer"),
                Err(e) => error!("Error flushing buffer for {}: {}", self.filename, e),
            };

            // Reset the last flush timer
            self.last_flush = Instant::now();
        }
        Ok(())
    }
}

impl FileSink {
    pub fn new(filename: String, flush_time_s: i32) -> std::io::Result<Self> {
        let file = std::fs::File::create(&filename)?; // Opens or creates the file
        let writer = std::io::BufWriter::new(file); // Wraps the file in BufWriter
        let flush_time = Duration::new(flush_time_s.try_into().unwrap(), 0);
        let last_flush = Instant::now();
        Ok(FileSink {
            filename,
            writer,
            flush_time,
            last_flush,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{BigEndian, ReadBytesExt};
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_sink_write() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let mut file_sink = FileSink::new(temp_path.clone(), 0).expect("Failed to create FileSink");

        let data = b"Hello, FileSink!".to_vec();

        file_sink.write(&data).expect("Failed to write data");

        let mut file = std::fs::File::open(temp_path).expect("Failed to open temp file");

        // Read the first 8 bytes to get the data size
        let data_size = file
            .read_u64::<BigEndian>()
            .expect("Failed to read data size");

        // Read the data
        let mut data_buf = vec![0u8; data_size as usize];
        file.read_exact(&mut data_buf).expect("Failed to read data");

        assert_eq!(data_buf, data);
    }
}
