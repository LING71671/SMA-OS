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

/// 任务检查点结构体 (用于暂停/恢复)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCheckpoint {
    pub version: u64,
    pub state_data: Vec<u8>,
    pub position: String,
    pub created_at: i64,
    pub metadata: Option<serde_json::Value>,
}

/// 任务生命周期事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum TaskEventType {
    TaskCreated,
    TaskStarted,
    TaskProgressUpdated { progress: f64 },
    TaskPaused { checkpoint: TaskCheckpoint },
    TaskResumed { from_checkpoint: bool },
    TaskCompleted,
    TaskFailed { error: String },
}

/// 任务事件包装结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent {
    pub event_id: Uuid,
    pub task_id: String,
    pub tenant_id: String,
    pub event_type: TaskEventType,
    pub timestamp: i64,
}
