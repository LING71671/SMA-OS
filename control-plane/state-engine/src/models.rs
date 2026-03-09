use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 核心状态事件结构体 (Event Sourcing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEvent {
    pub event_id: Uuid,
    pub tenant_id: String,
    pub namespace: String,
    pub version: u64,
    pub payload: serde_json::Value,
    pub timestamp: i64,
}

/// 快照结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub snapshot_id: Uuid,
    pub tenant_id: String,
    pub namespace: String,
    pub start_version: u64,
    pub end_version: u64,
    pub state_blob: serde_json::Value,
    pub created_at: i64,
}
