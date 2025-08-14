use anyhow::Result;
use rdkafka::producer::FutureRecord;
use rdkafka::producer::future_producer::Delivery;
use serde::Serialize;
use shared::config::{AppConfig, PartitioningMode};
use shared::create_producer_props;
use std::env;
use std::io::{self, BufRead};
use std::time::Duration;

#[derive(Serialize)]
struct Event {
    user_id: String,
    action: String,
    value: i64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args().skip_while(|a| a != "--config");
    let cfg_path = match args.next() {
        Some(_) => args
            .next()
            .unwrap_or_else(|| "labs/lab1_produce_consume/config_keyed.toml".into()),
        None => "labs/lab1_produce_consume/config_keyed.toml".into(),
    };

    let cfg = AppConfig::from_file(&cfg_path);

    let timeout = cfg.message_timeout_ms.unwrap_or(5000).to_string();
    let compression = cfg.compression.unwrap_or("lz4".to_string());
    let props = &[
        ("bootstrap.servers", cfg.bootstrap_servers),
        ("compression.type", compression),
        ("message.timeout.ms", timeout),
    ];

    let producer = create_producer_props(props)?;

    eprintln!(
        "Producer started with {:?} partitioning. Using config: {}",
        cfg.partitioning, cfg_path
    );
    eprintln!("Enter: user_id action value (e.g. u1 click 42). Ctrl+D to exit.");

    for line in io::stdin().lock().lines() {
        let line = line?;
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() < 3 {
            eprintln!("Format: user_id action value");
            continue;
        }

        let evt = Event {
            user_id: parts[0].to_string(),
            action: parts[1].to_string(),
            value: parts[2].parse().unwrap_or(0),
        };

        let payload = serde_json::to_vec(&evt)?;

        let record = match cfg.partitioning {
            PartitioningMode::Keyed => FutureRecord::to(&cfg.topic)
                .key(&evt.user_id)
                .payload(&payload),
            PartitioningMode::RoundRobin => {
                FutureRecord::to(&cfg.topic).payload(&payload) // NO KEY
            }
        };

        let delivery = producer.send(record, Duration::from_secs(0)).await;

        match delivery {
            Ok(Delivery {
                partition, offset, ..
            }) => eprintln!(
                "✅ Sent p{partition} @ {offset} key_mode={:?}",
                cfg.partitioning
            ),
            Err((e, _)) => eprintln!("❌ Delivery failed: {e}"),
        }
    }

    Ok(())
}
