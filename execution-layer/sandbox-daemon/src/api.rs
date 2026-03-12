use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn, error};

use crate::microvm::{
    MicroVMManager, VmConfig, VmStats, VmState, ResourceUsage,
    MicroVMError, FirecrackerVM, Snapshot,
};

/// API error response
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
}

impl From<MicroVMError> for ApiError {
    fn from(err: MicroVMError) -> Self {
        Self {
            error: "microvm_error".to_string(),
            message: err.to_string(),
        }
    }
}

/// API result type
pub type ApiResult<T> = Result<(StatusCode, Json<T>), (StatusCode, Json<ApiError>)>;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub manager: Arc<MicroVMManager>,
}

/// Create VM request
#[derive(Debug, Deserialize)]
pub struct CreateVmRequest {
    pub vcpu_count: Option<u8>,
    pub memory_mb: Option<u32>,
    pub kernel_image_path: Option<String>,
    pub rootfs_path: Option<String>,
    pub network_namespace: Option<String>,
}

/// Create VM response
#[derive(Debug, Serialize)]
pub struct CreateVmResponse {
    pub vm_id: String,
    pub state: String,
}

/// VM info response
#[derive(Debug, Serialize)]
pub struct VmInfoResponse {
    pub vm_id: String,
    pub state: String,
    pub socket_path: String,
    pub vcpu_count: u8,
    pub memory_mb: u32,
    pub pid: Option<u32>,
    pub created_at: String,
}

impl From<FirecrackerVM> for VmInfoResponse {
    fn from(vm: FirecrackerVM) -> Self {
        Self {
            vm_id: vm.vm_id,
            state: format!("{:?}", vm.state),
            socket_path: vm.socket_path,
            vcpu_count: vm.config.vcpu_count,
            memory_mb: vm.config.memory_mb,
            pid: vm.pid,
            created_at: vm.created_at.to_rfc3339(),
        }
    }
}

/// VM list response
#[derive(Debug, Serialize)]
pub struct VmListResponse {
    pub vms: Vec<VmInfoResponse>,
    pub total: usize,
}

/// VM stats response
#[derive(Debug, Serialize)]
pub struct VmStatsResponse {
    pub vm_id: String,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u32,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

/// Snapshot response
#[derive(Debug, Serialize)]
pub struct SnapshotResponse {
    pub snapshot_id: String,
    pub vm_id: String,
    pub created_at: String,
    pub path: String,
    pub size_bytes: u64,
}

impl From<Snapshot> for SnapshotResponse {
    fn from(snap: Snapshot) -> Self {
        Self {
            snapshot_id: snap.snapshot_id,
            vm_id: snap.vm_id,
            created_at: snap.created_at.to_rfc3339(),
            path: snap.path,
            size_bytes: snap.size_bytes,
        }
    }
}

/// Snapshot list response
#[derive(Debug, Serialize)]
pub struct SnapshotListResponse {
    pub snapshots: Vec<SnapshotResponse>,
    pub total: usize,
}

/// Generic success response
#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}

/// Create a new VM
pub async fn create_vm(
    State(state): State<AppState>,
    Json(req): Json<CreateVmRequest>,
) -> ApiResult<CreateVmResponse> {
    info!("[API] Creating new VM with request: {:?}", req);
    
    let mut config = VmConfig::default();
    
    if let Some(vcpu_count) = req.vcpu_count {
        config.vcpu_count = vcpu_count;
    }
    
    if let Some(memory_mb) = req.memory_mb {
        config.memory_mb = memory_mb;
    }
    
    if let Some(kernel_path) = req.kernel_image_path {
        config.kernel_image_path = kernel_path;
    }
    
    if let Some(rootfs_path) = req.rootfs_path {
        config.rootfs_path = rootfs_path;
    }
    
    if let Some(ns) = req.network_namespace {
        config.network_namespace = Some(ns);
    }
    
    match state.manager.create(config).await {
        Ok(vm_id) => {
            info!("[API] Created VM: {}", vm_id);
            Ok((
                StatusCode::CREATED,
                Json(CreateVmResponse {
                    vm_id: vm_id.clone(),
                    state: "Configured".to_string(),
                }),
            ))
        }
        Err(e) => {
            warn!("[API] Failed to create VM: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(e.into()),
            ))
        }
    }
}

/// List all VMs
pub async fn list_vms(
    State(state): State<AppState>,
) -> ApiResult<VmListResponse> {
    info!("[API] Listing all VMs");
    
    let vms = state.manager.list_vms().await;
    let vm_responses: Vec<VmInfoResponse> = vms.into_iter().map(VmInfoResponse::from).collect();
    
    Ok((
        StatusCode::OK,
        Json(VmListResponse {
            total: vm_responses.len(),
            vms: vm_responses,
        }),
    ))
}

/// Get VM by ID
pub async fn get_vm(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<VmInfoResponse> {
    info!("[API] Getting VM: {}", id);
    
    match state.manager.get_vm(&id).await {
        Ok(vm) => Ok((StatusCode::OK, Json(VmInfoResponse::from(vm)))),
        Err(e) => {
            let status = match e {
                MicroVMError::VmNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(e.into())))
        }
    }
}

/// Start a VM
pub async fn start_vm(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<SuccessResponse> {
    info!("[API] Starting VM: {}", id);
    
    match state.manager.start(&id).await {
        Ok(()) => Ok((
            StatusCode::OK,
            Json(SuccessResponse {
                success: true,
                message: format!("VM {} started successfully", id),
            }),
        )),
        Err(e) => {
            let status = match &e {
                MicroVMError::VmNotFound(_) => StatusCode::NOT_FOUND,
                MicroVMError::VmAlreadyRunning(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(e.into())))
        }
    }
}

/// Stop a VM
pub async fn stop_vm(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<SuccessResponse> {
    info!("[API] Stopping VM: {}", id);
    
    match state.manager.stop(&id).await {
        Ok(()) => Ok((
            StatusCode::OK,
            Json(SuccessResponse {
                success: true,
                message: format!("VM {} stopped successfully", id),
            }),
        )),
        Err(e) => {
            let status = match &e {
                MicroVMError::VmNotFound(_) => StatusCode::NOT_FOUND,
                MicroVMError::VmNotRunning(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(e.into())))
        }
    }
}

/// Destroy a VM
pub async fn destroy_vm(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<SuccessResponse> {
    info!("[API] Destroying VM: {}", id);
    
    match state.manager.destroy(&id).await {
        Ok(()) => Ok((
            StatusCode::OK,
            Json(SuccessResponse {
                success: true,
                message: format!("VM {} destroyed successfully", id),
            }),
        )),
        Err(e) => {
            let status = match &e {
                MicroVMError::VmNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(e.into())))
        }
    }
}

/// Get VM stats
pub async fn get_vm_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<VmStatsResponse> {
    info!("[API] Getting stats for VM: {}", id);
    
    match state.manager.get_vm_stats(&id).await {
        Ok(stats) => Ok((
            StatusCode::OK,
            Json(VmStatsResponse {
                vm_id: id,
                cpu_usage_percent: stats.cpu_usage_percent,
                memory_usage_mb: stats.memory_usage_mb,
                disk_read_bytes: stats.disk_read_bytes,
                disk_write_bytes: stats.disk_write_bytes,
                network_rx_bytes: stats.network_rx_bytes,
                network_tx_bytes: stats.network_tx_bytes,
            }),
        )),
        Err(e) => {
            let status = match &e {
                MicroVMError::VmNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(e.into())))
        }
    }
}

/// Create a snapshot of a VM
pub async fn create_snapshot(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<SnapshotResponse> {
    info!("[API] Creating snapshot for VM: {}", id);
    
    match state.manager.snapshot(&id).await {
        Ok(snapshot_id) => {
            // Get the snapshot details
            let snapshots = state.manager.list_snapshots().await;
            let snapshot = snapshots.into_iter()
                .find(|s| s.snapshot_id == snapshot_id)
                .ok_or_else(|| (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError {
                        error: "internal_error".to_string(),
                        message: "Snapshot created but not found".to_string(),
                    })
                ))?;
            
            Ok((
                StatusCode::CREATED,
                Json(SnapshotResponse::from(snapshot)),
            ))
        }
        Err(e) => {
            let status = match &e {
                MicroVMError::VmNotFound(_) => StatusCode::NOT_FOUND,
                MicroVMError::VmNotRunning(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(e.into())))
        }
    }
}

/// Restore a VM from a snapshot
pub async fn restore_snapshot(
    State(state): State<AppState>,
    Path(snapshot_id): Path<String>,
) -> ApiResult<CreateVmResponse> {
    info!("[API] Restoring from snapshot: {}", snapshot_id);
    
    match state.manager.restore(&snapshot_id).await {
        Ok(vm_id) => Ok((
            StatusCode::CREATED,
            Json(CreateVmResponse {
                vm_id,
                state: "Running".to_string(),
            }),
        )),
        Err(e) => {
            let status = match &e {
                MicroVMError::SnapshotNotFound(_) => StatusCode::NOT_FOUND,
                MicroVMError::ResourceLimit(_) => StatusCode::SERVICE_UNAVAILABLE,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(e.into())))
        }
    }
}

/// List all snapshots
pub async fn list_snapshots(
    State(state): State<AppState>,
) -> ApiResult<SnapshotListResponse> {
    info!("[API] Listing all snapshots");
    
    let snapshots = state.manager.list_snapshots().await;
    let snapshot_responses: Vec<SnapshotResponse> = snapshots
        .into_iter()
        .map(SnapshotResponse::from)
        .collect();
    
    Ok((
        StatusCode::OK,
        Json(SnapshotListResponse {
            total: snapshot_responses.len(),
            snapshots: snapshot_responses,
        }),
    ))
}

/// Delete a snapshot
pub async fn delete_snapshot(
    State(state): State<AppState>,
    Path(snapshot_id): Path<String>,
) -> ApiResult<SuccessResponse> {
    info!("[API] Deleting snapshot: {}", snapshot_id);
    
    match state.manager.delete_snapshot(&snapshot_id).await {
        Ok(()) => Ok((
            StatusCode::OK,
            Json(SuccessResponse {
                success: true,
                message: format!("Snapshot {} deleted successfully", snapshot_id),
            }),
        )),
        Err(e) => {
            let status = match &e {
                MicroVMError::SnapshotNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(e.into())))
        }
    }
}

/// Get resource usage
pub async fn get_resources(
    State(state): State<AppState>,
) -> ApiResult<ResourceUsage> {
    info!("[API] Getting resource usage");
    
    let usage = state.manager.get_total_resources().await;
    Ok((StatusCode::OK, Json(usage)))
}

/// Create the API router
pub fn create_router(manager: Arc<MicroVMManager>) -> Router {
    let state = AppState { manager };
    
    Router::new()
        // VM endpoints
        .route("/vms", post(create_vm).get(list_vms))
        .route("/vms/:id", get(get_vm).delete(destroy_vm))
        .route("/vms/:id/start", post(start_vm))
        .route("/vms/:id/stop", post(stop_vm))
        .route("/vms/:id/stats", get(get_vm_stats))
        .route("/vms/:id/snapshot", post(create_snapshot))
        // Snapshot endpoints
        .route("/snapshots", get(list_snapshots))
        .route("/snapshots/:snapshot_id", delete(delete_snapshot))
        .route("/snapshots/:snapshot_id/restore", post(restore_snapshot))
        // Resource endpoints
        .route("/resources", get(get_resources))
        .with_state(state)
}

/// Start the API server
pub async fn start_api_server(
    manager: Arc<MicroVMManager>,
    port: u16,
) -> anyhow::Result<()> {
    let app = create_router(manager);
    
    let addr = format!("0.0.0.0:{}", port);
    info!("[API] Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, Method};
    use tower::ServiceExt;
    
    fn create_test_manager() -> Arc<MicroVMManager> {
        Arc::new(MicroVMManager::new(10, "/tmp/test_snapshots".to_string()))
    }
    
    #[tokio::test]
    async fn test_create_vm_endpoint() {
        let manager = create_test_manager();
        let app = create_router(manager);
        
        let request_body = serde_json::json!({
            "vcpu_count": 2,
            "memory_mb": 512,
        });
        
        let request = Request::builder()
            .method(Method::POST)
            .uri("/vms")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }
    
    #[tokio::test]
    async fn test_list_vms_endpoint() {
        let manager = create_test_manager();
        let app = create_router(manager);
        
        let request = Request::builder()
            .method(Method::GET)
            .uri("/vms")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    #[tokio::test]
    async fn test_get_nonexistent_vm() {
        let manager = create_test_manager();
        let app = create_router(manager);
        
        let request = Request::builder()
            .method(Method::GET)
            .uri("/vms/non-existent")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
    
    #[tokio::test]
    async fn test_resources_endpoint() {
        let manager = create_test_manager();
        let app = create_router(manager);
        
        let request = Request::builder()
            .method(Method::GET)
            .uri("/resources")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
