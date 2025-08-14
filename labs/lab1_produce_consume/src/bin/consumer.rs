use std::env;

use anyhow::Result;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use serde::Deserialize;
use shared::config::AppConfig;
use shared::create_consumer_props;

#[derive(Deserialize, Debug)]
struct Event {
    user_id: String,
    action: String,
    value: i64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args();
    let mut cfg_path = "labs/lab1_produce_consume/config_keyed.toml".to_string();
    let mut group_override: Option<String> = None;

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
            _ => {}
        }
    }
    let cfg = AppConfig::from_file(&cfg_path);
    let group_id = group_override
        .as_deref()
        .or(cfg.group_id.as_deref())
        .unwrap_or("demo-consumer-group");

    let props = &[
        ("bootstrap.servers", cfg.bootstrap_servers),
        ("group.id", group_id.to_string()),
        ("enable.auto.commit", "true".to_string()),
        ("auto.offset.reset", "earliest".to_string()),
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
