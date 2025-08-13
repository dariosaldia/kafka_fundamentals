use anyhow::Result;
use rdkafka::config::ClientConfig;
use rdkafka::{consumer::StreamConsumer, producer::FutureProducer};

pub mod config;

pub fn create_producer(bootstrap: &str) -> Result<FutureProducer> {
    Ok(ClientConfig::new()
        .set("bootstrap.servers", bootstrap)
        .set("compression.type", "lz4")
        .set("message.timeout.ms", "5000")
        .create()?)
}

pub fn create_consumer(
    bootstrap: &str,
    group_id: &str,
    auto_commit: bool,
) -> Result<StreamConsumer> {
    Ok(ClientConfig::new()
        .set("bootstrap.servers", bootstrap)
        .set("group.id", group_id)
        .set(
            "enable.auto.commit",
            if auto_commit { "true" } else { "false" },
        )
        .set("auto.offset.reset", "earliest")
        .create()?)
}
