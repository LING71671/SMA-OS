//! Plugin executor interface
//!
//! Defines the contract for plugins that provide custom execution capabilities.

use crate::{Plugin, PluginConfig, PluginError, PluginHealth, PluginMetadata, ExecutionResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Executor plugin trait - for plugins that execute custom workloads
#[async_trait]
pub trait ExecutorPlugin: Plugin {
    /// Execute a task
    async fn execute(&self, context: ExecutionContext) -> Result<ExecutionResult, PluginError>;
    
    /// Cancel a running execution
    async fn cancel(&self, execution_id: Uuid) -> Result<(), PluginError>;
    
    /// Get execution status
    async fn status(&self, execution_id: Uuid) -> Result<ExecutionStatus, PluginError>;
    
    /// List active executions
    async fn active_executions(&self) -> Vec<ExecutionSummary>;
}

/// Execution context for plugin tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub execution_id: Uuid,
    pub task_id: String,
    pub tenant_id: String,
    pub namespace: String,
    pub action_name: String,
    pub payload: serde_json::Value,
    pub config: HashMap<String, serde_json::Value>,
    pub timeout_secs: u64,
}

impl ExecutionContext {
    pub fn new(
        task_id: String,
        tenant_id: String,
        namespace: String,
        action_name: String,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            execution_id: Uuid::new_v4(),
            task_id,
            tenant_id,
            namespace,
            action_name,
            payload,
            config: Default::default(),
            timeout_secs: 300,
        }
    }
    
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }
    
    pub fn with_config(mut self, config: HashMap<String, serde_json::Value>) -> Self {
        self.config = config;
        self
    }
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub execution_id: Uuid,
    pub status: ExecutionStatus,
    pub output: Option<serde_json::Value>,
    pub logs: Vec<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub resource_usage: ResourceUsage,
}

impl ExecutionResult {
    pub fn success(execution_id: Uuid, output: serde_json::Value) -> Self {
        Self {
            execution_id,
            status: ExecutionStatus::Completed,
            output: Some(output),
            logs: vec![],
            started_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
            resource_usage: Default::default(),
        }
    }
    
    pub fn failed(execution_id: Uuid, error: String) -> Self {
        Self {
            execution_id,
            status: ExecutionStatus::Failed(error),
            output: None,
            logs: vec![error.clone()],
            started_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
            resource_usage: Default::default(),
        }
    }
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Running { started_at: chrono::DateTime<chrono::Utc> },
    Completed,
    Failed(String),
    Cancelled,
    Timeout,
}

impl ExecutionStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, 
            ExecutionStatus::Completed |
            ExecutionStatus::Failed(_) |
            ExecutionStatus::Cancelled |
            ExecutionStatus::Timeout
        )
    }
    
    pub fn is_running(&self) -> bool {
        matches!(self, ExecutionStatus::Running { .. })
    }
}

/// Execution summary for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub execution_id: Uuid,
    pub task_id: String,
    pub status: ExecutionStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceUsage {
    pub cpu_seconds: f64,
    pub memory_mb: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

/// Default executor implementation
pub struct DefaultExecutor {
    metadata: PluginMetadata,
}

impl DefaultExecutor {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "default-executor".to_string(),
                version: crate::PluginVersion::new(1, 0, 0),
                description: "Default executor for standard workloads".to_string(),
                author: "SMA-OS".to_string(),
                license: "MIT".to_string(),
                homepage: None,
                repository: None,
                keywords: vec!["executor".to_string()],
                categories: vec!["core".to_string()],
            },
        }
    }
}

#[async_trait]
impl Plugin for DefaultExecutor {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }
    
    async fn init(&mut self, _config: PluginConfig) -> Result<(), PluginError> {
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
    
    async fn health(&self) -> PluginHealth {
        PluginHealth {
            status: crate::HealthStatus::Healthy,
            last_check: chrono::Utc::now(),
            message: None,
            metrics: Default::default(),
        }
    }
    
    fn capabilities(&self) -> Vec<crate::PluginCapability> {
        vec![crate::PluginCapability::Executor {
            runtime: crate::ExecutorRuntime::Native,
            config: Default::default(),
        }]
    }
}

#[async_trait]
impl ExecutorPlugin for DefaultExecutor {
    async fn execute(&self, context: ExecutionContext) -> Result<ExecutionResult, PluginError> {
        // Default implementation - just echo back the payload
        let result = ExecutionResult::success(
            context.execution_id,
            serde_json::json!({
                "task_id": context.task_id,
                "action": context.action_name,
                "echo": context.payload,
            })
        );
        
        Ok(result)
    }
    
    async fn cancel(&self, _execution_id: Uuid) -> Result<(), PluginError> {
        // No-op for default executor
        Ok(())
    }
    
    async fn status(&self, _execution_id: Uuid) -> Result<ExecutionStatus, PluginError> {
        Ok(ExecutionStatus::Completed)
    }
    
    async fn active_executions(&self) -> Vec<ExecutionSummary> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context() {
        let context = ExecutionContext::new(
            "task-1".to_string(),
            "tenant-1".to_string(),
            "default".to_string(),
            "test-action".to_string(),
            serde_json::json!({"key": "value"}),
        );
        
        assert_eq!(context.task_id, "task-1");
        assert_eq!(context.timeout_secs, 300); // Default
    }

    #[test]
    fn test_execution_status() {
        assert!(ExecutionStatus::Completed.is_terminal());
        assert!(ExecutionStatus::Failed("error".to_string()).is_terminal());
        assert!(!ExecutionStatus::Pending.is_terminal());
        assert!(!ExecutionStatus::Running { started_at: chrono::Utc::now() }.is_terminal());
    }

    #[test]
    fn test_execution_result() {
        let id = Uuid::new_v4();
        let result = ExecutionResult::success(id, serde_json::json!({"status": "ok"}));
        
        assert_eq!(result.execution_id, id);
        assert_eq!(result.status, ExecutionStatus::Completed);
        assert!(result.output.is_some());
    }
}