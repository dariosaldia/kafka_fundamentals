use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Event {
    pub user_id: String,
    pub action: String,
    pub value: i64,
}
