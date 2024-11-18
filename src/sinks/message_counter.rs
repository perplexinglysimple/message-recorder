use crate::sink::{Sink, SinkError};

use getset::Getters;

#[derive(Debug, Getters)]
pub struct MessageCounter {
    #[get = "pub"]
    message_count: u64,
}

impl Sink for MessageCounter {
    fn write(&mut self, _: &Vec<u8>) -> Result<(), SinkError> {
        self.message_count += 1;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), SinkError> {
        Ok(())
    }
}

impl MessageCounter {
    pub fn new() -> MessageCounter {
        MessageCounter { message_count: 0 }
    }

    pub fn clear_message_count(&mut self) {
        self.message_count = 0;
    }
}
