use std::env;

use anyhow::Result;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use shared::config::AppConfig;
use shared::create_consumer_props;
use shared::event::Event;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args();
    let mut cfg_path = "shared/config.toml".to_string();
    let mut group_override: Option<String> = None;
    let mut profile = "lab1.keyed".to_string();

    while let Some(a) = args.next() {
        match a.as_str() {
            "--config" => {
                if let Some(p) = args.next() {
                    cfg_path = p;
                }
            }
            "--group-id" => {
                if let Some(g) = args.next() {
                    group_override = Some(g);
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
    let group_id = group_override
        .as_deref()
        .or(cfg.group_id.as_deref())
        .unwrap_or("demo-consumer-group");

    let props = &[
        ("bootstrap.servers", cfg.bootstrap_servers),
        ("group.id", group_id.to_string()),
        ("enable.auto.commit", cfg.enable_auto_commit.to_string()),
        ("auto.offset.reset", cfg.auto_offset_reset.to_string()),
    ];

    let consumer: StreamConsumer = create_consumer_props(props)?;
    consumer.subscribe(&[&cfg.topic])?;
    eprintln!(
        "Consumer using config: {cfg_path} | group='{group_id}' | topic='{}'",
        cfg.topic
    );

    loop {
        match consumer.recv().await {
            Err(e) => eprintln!("Read error: {e}"),
            Ok(m) => {
                let key = m.key().and_then(|k| std::str::from_utf8(k).ok());
                let payload = m.payload().unwrap_or_default();
                let partition = m.partition();
                let offset = m.offset();

                match serde_json::from_slice::<Event>(payload) {
                    Ok(ev) => println!(
                        "partition={partition} @ offset={offset} key={:?} => {:?}",
                        key, ev
                    ),
                    Err(_) => eprintln!("Non-JSON message p{partition} @ {offset} key={:?}", key),
                }
            }
        }
    }
}
