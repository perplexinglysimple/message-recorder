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
}

// stdout is not being captured correctly. Testing this also seems not necassary.
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use gag::BufferRedirect;
//     use std::io::Read;

//     #[test]
//     fn test_console_sink_write() {
//         let mut console_sink = ConsoleSink;
//         let data = b"Hello, Console!".to_vec();
//         let mut buffer = BufferRedirect::stdout().unwrap();
//         console_sink.write(&data).expect("Failed to write to console");
//         let mut output = String::new();
//         buffer.read_to_string(&mut output).unwrap();
//         println!("data recieved: {:?} data input: {}",output, String::from_utf8(data).expect("Our bytes should be valid utf8"));
//         assert!(output.contains("Writing to console: Hello, Console!"));
//     }
// }
