use anyhow::Result;
use rdkafka::producer::future_producer::Delivery;
use shared::config::AppConfig;
use shared::create_producer_props;
use shared::event::Event;
use shared::record::create_future_record;
use std::env;
use std::io::{self, BufRead};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args();
    let mut cfg_path = "shared/config.toml".to_string();
    let mut profile = "lab2.default".to_string();

    while let Some(a) = args.next() {
        match a.as_str() {
            "--config" => {
                if let Some(p) = args.next() {
                    cfg_path = p;
                }
            }
            "--profile" => {
                if let Some(p) = args.next() {
                    profile = p;
                }
            }
            _ => {}
        }
    }

    let cfg = AppConfig::from_file(&cfg_path, &profile);

    let timeout = cfg.message_timeout_ms.unwrap_or(5000).to_string();
    let compression = cfg.compression.unwrap_or("lz4".to_string());
    let props = &[
        ("bootstrap.servers", cfg.bootstrap_servers),
        ("compression.type", compression),
        ("message.timeout.ms", timeout),
    ];

    let producer = create_producer_props(props)?;

    eprintln!(
        "Producer (Lab 2) using {cfg_path} | partitioning={:?}",
        cfg.partitioning
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

        let record =
            create_future_record(Some(&evt.user_id), &payload, &cfg.topic, cfg.partitioning)?;

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
