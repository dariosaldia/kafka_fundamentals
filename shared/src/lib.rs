use anyhow::Result;
use rdkafka::config::ClientConfig;
use rdkafka::{consumer::StreamConsumer, producer::FutureProducer};

pub mod config;
pub mod event;
pub mod record;

pub fn create_producer_props<K, V>(props: &[(K, V)]) -> Result<FutureProducer>
where
    K: AsRef<str>,
    V: AsRef<str>,
{
    let mut cfg = ClientConfig::new();
    for (k, v) in props {
        cfg.set(k.as_ref(), v.as_ref());
    }
    Ok(cfg.create()?)
}

pub fn create_consumer_props<K, V>(props: &[(K, V)]) -> Result<StreamConsumer>
where
    K: AsRef<str>,
    V: AsRef<str>,
{
    let mut cfg = ClientConfig::new();
    for (k, v) in props {
        cfg.set(k.as_ref(), v.as_ref());
    }
    Ok(cfg.create()?)
}
