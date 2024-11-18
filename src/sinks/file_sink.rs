use byteorder::{BigEndian, WriteBytesExt};
use getset::Getters;
use log::info;

use crate::sink::{Sink, SinkError};
use crate::sinks::raw_file_sink::RawFileSink;

#[derive(Debug, Getters)]
pub struct FileSink {
    file_handle: RawFileSink,
}

impl Sink for FileSink {
    fn write(&mut self, data: &Vec<u8>) -> Result<(), SinkError> {
        info!(
            "Writing to file {}: {:?}",
            self.file_handle.filename(),
            data.clone()
        );
        let mut data_size_vec = vec![];
        WriteBytesExt::write_u64::<BigEndian>(&mut data_size_vec, data.len() as u64)?;
        self.file_handle.write(&data_size_vec)?;
        self.file_handle.write(data)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), SinkError> {
        self.file_handle.flush()?;
        Ok(())
    }
}

impl FileSink {
    pub fn new(filename: String, flush_time_s: i32) -> std::io::Result<Self> {
        let file_handle = RawFileSink::new(filename, flush_time_s)?;
        Ok(FileSink {
            file_handle: file_handle,
        })
    }

    pub fn filename(&self) -> &String {
        self.file_handle.filename()
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
