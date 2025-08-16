use std::fs::read_to_string;

use serde::Deserialize;
use toml::{
    Value::{self, Table},
    from_str, to_string,
};

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum PartitioningMode {
    Keyed,
    RoundRobin,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub bootstrap_servers: String,
    pub topic: String,
    pub group_id: Option<String>,
    pub compression: Option<String>,
    pub message_timeout_ms: Option<u64>,
    pub auto_offset_reset: String,
    pub enable_auto_commit: bool,
    pub enable_auto_offset_store: Option<bool>,
    #[serde(default = "default_partitioning_mode")]
    pub partitioning: PartitioningMode,
}

fn default_partitioning_mode() -> PartitioningMode {
    PartitioningMode::Keyed
}

impl AppConfig {
    pub fn from_file(path: &str, profile: &str) -> Self {
        let text = read_to_string(path).expect("Config file not found");
        let full: Value = from_str(&text).expect("Invalid TOML");

        let common = full
            .get("common")
            .cloned()
            .unwrap_or(Table(Default::default()));

        let profile_val = get_profile(&full, profile)
            .cloned()
            .unwrap_or(Table(Default::default()));

        let merged = merge_tables(common, profile_val);
        from_str(&to_string(&merged).unwrap()).expect("Cannot parse merged config")
    }
}

fn get_profile<'a>(root: &'a Value, profile_path: &str) -> Option<&'a Value> {
    profile_path
        .split('.')
        .fold(Some(root), |acc, key| acc?.get(key))
}

// Merge two TOML tables (right overrides left)
fn merge_tables(base: Value, overlay: Value) -> Value {
    match (base, overlay) {
        (Table(mut b), Table(o)) => {
            for (k, v) in o {
                b.insert(k, v);
            }
            Table(b)
        }
        (_, o) => o,
    }
}
