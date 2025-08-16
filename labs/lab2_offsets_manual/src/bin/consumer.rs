use std::env;
use std::time::Duration;

use anyhow::Result;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use shared::config::AppConfig;
use shared::create_consumer_props;
use shared::event::Event;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args();
    let mut cfg_path = "shared/config.toml".to_string();
    let mut profile = "lab2.keyed".to_string();
    let mut group_override: Option<String> = None;
    let mut fail_mod: i64 = 5; // fail when value % fail_mod == 0 (if >0)
    let mut fail_action: Option<String> = None; // also fail if action matches

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
            "--fail-mod" => {
                if let Some(m) = args.next() {
                    fail_mod = m.parse().unwrap_or(0);
                }
            }
            "--fail-action" => {
                if let Some(act) = args.next() {
                    fail_action = Some(act);
                }
            }
            _ => {}
        }
    }
    let cfg = AppConfig::from_file(&cfg_path, &profile);
    let group_id = group_override
        .as_deref()
        .or(cfg.group_id.as_deref())
        .unwrap_or("lab2-consumer-group");

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
    ];
    let consumer: StreamConsumer = create_consumer_props(props)?;
    consumer.subscribe(&[&cfg.topic])?;
    eprintln!(
        "Lab 2 consumer | cfg={cfg_path} | group='{group_id}' | topic='{}'",
        cfg.topic
    );
    eprintln!(
        "Fail rules: value % {} == 0 {}",
        fail_mod,
        fail_action
            .as_ref()
            .map(|a| format!("or action == '{a}'"))
            .unwrap_or_default()
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
                    Err(_) => {
                        eprintln!("❌ Non-JSON p{partition} @ {offset} key={key:?} -> NO COMMIT")
                    }
                    Ok(ev) => {
                        let mut fail = false;
                        if fail_mod > 0 && ev.value % fail_mod == 0 {
                            fail = true;
                        }
                        if let Some(a) = &fail_action {
                            if ev.action == *a {
                                fail = true;
                            }
                        }

                        if fail {
                            eprintln!(
                                "❌ Simulated failure p{partition} @ {offset} key={key:?} => {:?}",
                                ev
                            );
                            sleep(Duration::from_millis(200)).await;
                        } else {
                            consumer.store_offset(&cfg.topic, partition, offset)?; // mark as processed
                            consumer.commit_message(&m, CommitMode::Sync)?; // commit offset
                            println!("✅ COMMIT p{partition} @ {offset} key={key:?} => {:?}", ev);
                        }
                    }
                }
            }
        }
    }
}
