use std::env;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::message::Message;
use shared::config::AppConfig;
use shared::create_consumer_props;
use shared::event::Event;
use tokio::time::sleep;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CommitModeCli {
    Pre,  // commit before processing (at-most-once pattern)
    Post, // commit after processing (at-least-once pattern)
    Txn,  // placeholder (transactions), not fully implemented in this lab
}

fn parse_commit_mode(s: &str) -> CommitModeCli {
    match s {
        "pre" => CommitModeCli::Pre,
        "post" => CommitModeCli::Post,
        "txn" | "transactional" => CommitModeCli::Txn,
        _ => CommitModeCli::Post,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args();
    let mut cfg_path = "shared/config.toml".to_string();
    let mut profile = "lab4.atleastonce".to_string();
    let mut group_override: Option<String> = None;
    let mut commit_mode = CommitModeCli::Post;
    let mut fail_mod: i64 = 0; // 0 disables failure-by-mod
    let mut crash_after: i64 = -1; // -1 disables crash-after counter

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
            "--commit-mode" => {
                if let Some(cm) = args.next() {
                    commit_mode = parse_commit_mode(&cm);
                }
            }
            "--fail-mod" => {
                if let Some(fm) = args.next() {
                    fail_mod = fm.parse().unwrap_or(0);
                }
            }
            "--crash-after" => {
                if let Some(ca) = args.next() {
                    crash_after = ca.parse().unwrap_or(-1);
                }
            }
            _ => {}
        }
    }
    let cfg = AppConfig::from_file(&cfg_path, &profile);
    let group_id = group_override
        .as_deref()
        .or(cfg.group_id.as_deref())
        .unwrap_or("lab4-consumer-group");

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
    let consumer = Arc::new(create_consumer_props(props)?);
    consumer.subscribe(&[&cfg.topic])?;

    eprintln!(
        "Lab4 consumer| profile={profile} | group={group_id} | topic='{}' | mode={commit_mode:?} | fail_mod={fail_mod} | crash_after={crash_after}",
        cfg.topic
    );

    let mut processed_ok_count: i64 = 0;

    loop {
        match consumer.recv().await {
            Err(e) => eprintln!("Read error: {e}"),
            Ok(m) => {
                let key = m.key().and_then(|k| std::str::from_utf8(k).ok());
                let payload = m.payload().unwrap_or_default();
                let p = m.partition();
                let o = m.offset();

                // Crash switch (before doing anything else): simulate "crash after N successes"
                if crash_after >= 0
                    && processed_ok_count >= crash_after
                    && commit_mode == CommitModeCli::Post
                {
                    eprintln!(
                        "💥 CRASHING BEFORE COMMIT (post) after {processed_ok_count} processed message(s)"
                    );
                    std::process::exit(1);
                }

                // Decode
                let ev = match serde_json::from_slice::<Event>(payload) {
                    Ok(ev) => ev,
                    Err(_) => {
                        eprintln!("❌ Non-JSON p{p} @ {o} key={key:?} -> skipping");
                        continue;
                    }
                };

                // Decide if this message should "fail"
                let should_fail = fail_mod > 0 && ev.value % fail_mod == 0;

                match commit_mode {
                    CommitModeCli::Pre => {
                        // At-most-once: commit first, then process
                        consumer.store_offset(m.topic(), m.partition(), m.offset())?;
                        consumer.commit_message(&m, CommitMode::Sync)?;
                        eprintln!("✅ COMMIT (pre) p{p} @ {o}");
                        // Now "process"
                        if should_fail {
                            eprintln!("❌ PROCESSING FAILED p{p} @ {o} key={key:?} => {:?}", ev);
                            // already committed -> message won't be redelivered (loss)
                        } else {
                            processed_ok_count += 1;
                            println!("✅ PROCESSED (pre) p{p} @ {o} key={key:?} => {:?}", ev);
                        }
                    }
                    CommitModeCli::Post => {
                        // At-least-once: process first, then commit
                        if should_fail {
                            eprintln!("❌ PROCESSING FAILED p{p} @ {o} key={key:?} => {:?}", ev);
                            // do NOT store/commit -> message will be redelivered
                            // small delay to make logs readable
                            sleep(Duration::from_millis(150)).await;
                        } else {
                            processed_ok_count += 1;
                            println!("✅ PROCESSED (post) p{p} @ {o} key={key:?} => {:?}", ev);
                            consumer.store_offset(m.topic(), m.partition(), m.offset())?;
                            consumer.commit_message(&m, CommitMode::Sync)?;
                            eprintln!("✅ COMMIT (post) p{p} @ {o}");
                        }
                    }
                    CommitModeCli::Txn => {
                        // Placeholder: true EOS requires transactional producer and
                        // sending offsets to the transaction with group metadata.
                        // Kept as a stub for now.
                        println!(
                            "(txn mode not fully implemented in this lab) p{p} @ {o} key={key:?} => {:?}",
                            ev
                        );
                        // Behave like post for demo:
                        consumer.store_offset(m.topic(), m.partition(), m.offset())?;
                        consumer.commit_message(&m, CommitMode::Sync)?;
                    }
                }
            }
        }
    }
}
