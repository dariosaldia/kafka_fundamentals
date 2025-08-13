use serde::Deserialize;

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
    #[serde(default = "default_partitioning_mode")]
    pub partitioning: PartitioningMode,
}

fn default_partitioning_mode() -> PartitioningMode {
    PartitioningMode::Keyed
}

impl AppConfig {
    pub fn from_file(path: &str) -> Self {
        config::Config::builder()
            .add_source(config::File::with_name(path))
            .build()
            .expect("Failed to load config file")
            .try_deserialize()
            .expect("Invalid config format")
    }
}
