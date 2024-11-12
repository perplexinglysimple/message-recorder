use std::io::Write;
use std::time::{Duration, Instant};

use byteorder::{BigEndian, WriteBytesExt};
use log::{debug, error, info};

use crate::sink::{Sink, SinkError};
use crate::sinks::file_sink::FileSink;

use flate2::write::GzEncoder;
use flate2::Compression;

#[derive(Debug)]
pub struct CompressedFileSink {
    filename: String,
    file_sink: FileSink,
    compression_level: i32,
}

impl Sink for CompressedFileSink {
    fn write(&mut self, data: &Vec<u8>) -> Result<(), SinkError> {
        // let mut encoder = GzEncoder::new(writer, Compression::new(self.compression_level as u32));    
        self.file_sink.write(data)
    }
}

impl CompressedFileSink {
    pub fn new(filename: String, flush_time_s: i32, compression_level: i32) -> std::io::Result<Self> {  
        Ok(CompressedFileSink {
            filename: filename,
            file_sink: FileSink::new(filename, flush_time_s)?,
            compression_level: compression_level,
        })
    }
}

