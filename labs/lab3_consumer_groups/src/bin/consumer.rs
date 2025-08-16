use std::env;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use rdkafka::consumer::Consumer;
use rdkafka::message::Message;
use shared::config::AppConfig;
use shared::create_consumer_props;
use shared::event::Event;
use tokio::time::sleep;

fn instance_id() -> String {
    // Allow override via ENV; otherwise use PID for uniqueness
    if let Ok(id) = env::var("INSTANCE_ID") {
        return id;
    }
    format!("lab3-consumer--{}", std::process::id())
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args();
    let mut cfg_path = "shared/config.toml".to_string();
    let mut profile = "lab3.default".to_string();
    let mut group_override: Option<String> = None;

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
            "--group-id" => {
                if let Some(g) = args.next() {
                    group_override = Some(g);
                }
            }
            _ => {}
        }
    }
    let cfg = AppConfig::from_file(&cfg_path, &profile);
    let group_id = group_override
        .as_deref()
        .or(cfg.group_id.as_deref())
        .unwrap_or("lab3-consumer-group");
    let id = instance_id();

    let enable_auto_offset_store = cfg
        .enable_auto_offset_store
        .expect("enable_auto_offset_store config was expected");

    let props = &[
        ("bootstrap.servers", cfg.bootstrap_servers),
        ("group.id", group_id.to_string()),
        ("enable.auto.commit", cfg.enable_auto_commit.to_string()),
        (
            "enable.auto.offset.store",
            enable_auto_offset_store.to_string(),
        ),
        ("auto.offset.reset", cfg.auto_offset_reset),
        ("client.id", id.clone()),
    ];
    let consumer = Arc::new(create_consumer_props(props)?);
    consumer.subscribe(&[&cfg.topic])?;

    eprintln!(
        "[{id}] started | profile={profile} | group={group_id} | topic='{}'",
        cfg.topic
    );

    // Periodically print current assignment so rebalances are visible.
    let id2 = id.clone();
    let consumer_for_task = Arc::clone(&consumer);
    tokio::spawn(async move {
        loop {
            match consumer_for_task.assignment() {
                Ok(tp_list) => {
                    let parts: Vec<String> = tp_list
                        .elements()
                        .into_iter()
                        .map(|tp| format!("p{}", tp.partition()))
                        .collect();
                    eprintln!("[{id2}] assignment: [{}]", parts.join(", "));
                }
                Err(e) => eprintln!("[{id2}] assignment error: {e}"),
            }
            sleep(Duration::from_secs(5)).await;
        }
    });

    loop {
        match consumer.recv().await {
            Err(e) => eprintln!("[{id}] read error: {e}"),
            Ok(m) => {
                let key = m.key().and_then(|k| std::str::from_utf8(k).ok());
                let payload = m.payload().unwrap_or_default();
                let p = m.partition();
                let o = m.offset();

                match serde_json::from_slice::<Event>(payload) {
                    Ok(ev) => println!("[{id}] p{p} @ {o} key={key:?} => {:?}", ev),
                    Err(_) => eprintln!("[{id}] Non-JSON p{p} @ {o} key={key:?}"),
                }
            }
        }
    }
}
