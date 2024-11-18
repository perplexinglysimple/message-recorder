use std::io::Write;

use log::{debug, error};

use crate::sink::{Sink, SinkError};
use crate::sinks::file_sink::FileSink;

use flate2::write::GzEncoder;
use flate2::Compression;
use getset::Getters;

#[derive(Debug, Getters)]
pub struct CompressedFileSink {
    file_sink: FileSink,
    #[get = "pub"]
    compression_level: i32,
}

impl Sink for CompressedFileSink {
    fn write(&mut self, data: &Vec<u8>) -> Result<(), SinkError> {
        let mut _encoder =
            GzEncoder::new(Vec::new(), Compression::new(self.compression_level as u32));
        _encoder.write_all(data)?;
        match _encoder.finish() {
            Ok(res) => {
                debug!(
                    "Compressed the message from size {} to size {}",
                    data.len(),
                    res.len()
                );
                self.file_sink.write(&res)
            }
            Err(e) => {
                error!("Failed with error {}", e);
                Err(SinkError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to compress data",
                )))
            }
        }
    }

    fn flush(&mut self) -> Result<(), SinkError> {
        self.file_sink.flush()
    }
}

impl CompressedFileSink {
    pub fn new(
        filename: String,
        flush_time_s: i32,
        compression_level: i32,
    ) -> std::io::Result<Self> {
        if (compression_level as u32) < Compression::level(&Compression::fast())
            || (compression_level as u32) > Compression::level(&Compression::best())
        {
            error!("Failed to create the CompressedFileSink with filename:{}, flush_time_s:{}, compression_leve:{}", filename, flush_time_s, compression_level);
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Bad compression value",
            ));
        }
        let f_sink = FileSink::new(filename.clone(), flush_time_s)?;
        Ok(CompressedFileSink {
            file_sink: f_sink,
            compression_level: compression_level,
        })
    }

    pub fn filename(&self) -> &String {
        self.file_sink.filename()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_compressed_file_sink_creation() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.gz");
        let file_path_str = file_path.to_str().unwrap().to_string();

        let sink = CompressedFileSink::new(file_path_str.clone(), 5, 5);
        assert!(sink.is_ok(), "Failed to create CompressedFileSink");

        let sink = sink.unwrap();
        assert_eq!(sink.filename(), &file_path_str);
    }

    #[test]
    fn test_compressed_file_sink_write_success() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.gz");
        let file_path_str = file_path.to_str().unwrap().to_string();

        let mut sink = CompressedFileSink::new(file_path_str.clone(), 5, 5).unwrap();

        let data = b"Hello, world!".to_vec();
        let write_result = sink.write(&data);
        assert!(
            write_result.is_ok(),
            "Write operation failed on CompressedFileSink"
        );
        let flush_result = sink.flush();
        assert!(
            flush_result.is_ok(),
            "Flush operation failed on the CompressedFileSink"
        );

        // Verify that the data was written and compressed
        let file_contents = fs::read(file_path).expect("Failed to read the compressed file");
        println!("data: {:?} compressed data: {:?}", data, file_contents);
        assert!(
            !file_contents.is_empty(),
            "File should not be empty after write"
        );
    }

    #[test]
    fn test_compressed_file_sink_creation_invalid_compression_low() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.gz");
        let file_path_str = file_path.to_str().unwrap().to_string();

        // Using an invalid compression level (too low)
        let invalid_compression_level = (Compression::level(&Compression::fast()) - 1) as i32;
        let sink = CompressedFileSink::new(file_path_str.clone(), 5, invalid_compression_level);

        assert!(
            sink.is_err(),
            "CompressedFileSink should fail with compression level too low"
        );
    }

    #[test]
    fn test_compressed_file_sink_creation_invalid_compression_high() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.gz");
        let file_path_str = file_path.to_str().unwrap().to_string();

        // Using an invalid compression level (too high)
        let invalid_compression_level = (Compression::level(&Compression::best()) + 1) as i32;
        let sink = CompressedFileSink::new(file_path_str.clone(), 5, invalid_compression_level);

        assert!(
            sink.is_err(),
            "CompressedFileSink should fail with compression level too high"
        );
    }

    #[test]
    fn test_compressed_file_sink_file_write_failure() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("nonexistent_dir/test_file.gz");
        let file_path_str = file_path.to_str().unwrap().to_string();

        // Expect file sink creation to fail due to invalid path
        let sink = CompressedFileSink::new(file_path_str.clone(), 5, 5);
        assert!(
            sink.is_err(),
            "CompressedFileSink creation should fail with invalid path"
        );
    }
}
