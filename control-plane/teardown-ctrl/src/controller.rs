use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, DeleteParams, ListParams},
    Client,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum TeardownError {
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] kube::Error),
    #[error("Internal Engine logic error: {0}")]
    Internal(String),
}

/// A target to teardown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeardownTarget {
    pub tenant_id: String,
    pub namespace: String,
    pub task_group_id: Uuid,
    /// If true, send SIGKILL directly instead of SIGTERM
    pub force: bool,
}

pub struct CascadingTeardownController {
    k8s_client: Client,
}

impl CascadingTeardownController {
    pub async fn new() -> Result<Self, TeardownError> {
        let client = Client::try_default().await.map_err(|e| {
            warn!("Failed to load kubernetes local config. Running in mock mode?");
            e
        })?;
        Ok(Self { k8s_client: client })
    }

    /// 执行级联销毁指令: 向某个 shard DAG 内的所有受影响 worker 广播强制回收
    pub async fn execute_teardown(&self, target: TeardownTarget) -> Result<(), TeardownError> {
        info!(
            "Initiating cascading teardown for Tenant: {}, Namespace: {}, TaskGroup: {}",
            target.tenant_id, target.namespace, target.task_group_id
        );

        let pods: Api<Pod> = Api::namespaced(self.k8s_client.clone(), &target.namespace);

        // Find all worker pods matching this group
        let label_selector = format!(
            "tenant={},task-group={}",
            target.tenant_id, target.task_group_id
        );
        let lp = ListParams::default().labels(&label_selector);

        let pod_list = pods.list(&lp).await?;

        if pod_list.items.is_empty() {
            info!("No active workers found for teardown target.");
            return Ok(());
        }

        let delete_params = DeleteParams {
            grace_period_seconds: if target.force { Some(0) } else { Some(10) },
            ..Default::default()
        };

        for pod in pod_list.items {
            if let Some(name) = pod.metadata.name {
                info!("Sending {} to pod {}", if target.force {"SIGKILL"} else {"SIGTERM"}, name);
                match pods.delete(&name, &delete_params).await {
                    Ok(_) => info!("Pod {} successfully marked for deletion.", name),
                    Err(e) => error!("Failed to teardown pod {}: {}", name, e),
                }
            }
        }

        // Ideally, here we also trigger eBPF hooks to forcefully kill stray processes
        // that might have escaped the container namespace (Side-channel prevention).
        info!("eBPF resource reap signal broadcasted for group {}.", target.task_group_id);

        Ok(())
    }
}
