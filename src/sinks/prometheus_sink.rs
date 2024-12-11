use log::info;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::registry::Registry;

use crate::sink::{Sink, SinkError};

#[derive(Debug)]
pub struct PrometheusSink {
    data_received: Counter,
    messages_received: Counter,
}

impl PrometheusSink {
    pub fn new(registry: &mut Registry, endpoint_name: &str) -> Self {
        let data_received = Counter::default();
        let messages_received = Counter::default();

        // Register the metrics with the provided Prometheus registry
        registry.register(
            format!("{}_data_received", endpoint_name),
            "Total amount of data received (in bytes)",
            data_received.clone(),
        );
        registry.register(
            format!("{}_messages_received", endpoint_name),
            "Total number of messages received",
            messages_received.clone(),
        );

        PrometheusSink {
            data_received,
            messages_received,
        }
    }
}

impl Sink for PrometheusSink {
    fn write(&mut self, data: &Vec<u8>) -> Result<(), SinkError> {
        // Update metrics
        self.data_received.inc_by(data.len() as u64);
        self.messages_received.inc();

        info!(
            "PrometheusSink: Received {} bytes in message {}.",
            data.len(),
            self.messages_received.get()
        );

        Ok(())
    }

    fn flush(&mut self) -> Result<(), SinkError> {
        Ok(())
    }
}
