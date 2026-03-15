//! gRPC service implementation for StateEngine
//!
//! Implements the StateEngine gRPC service defined in proto/state_engine.proto,
//! providing AppendEvent, GetEvents, and HealthCheck RPCs.

use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{info, warn};

use crate::engine::StateEngine;
use crate::failover::{FailoverManager, HealthStatus};
use crate::limiter::RateLimiterRegistry;

/// Maximum payload size (1MB) to prevent DoS attacks
const MAX_PAYLOAD_SIZE: usize = 1_048_576;

/// Generated protobuf types from state_engine.proto
pub mod proto {
    tonic::include_proto!("state_engine");
}

use proto::state_engine_server::StateEngineServer as GrpcStateEngineServer;
use proto::state_engine_server::StateEngine as StateEngineRpc;

/// gRPC service wrapper that holds the StateEngine and integration components
pub struct StateEngineService {
    engine: Arc<StateEngine>,
    failover: Arc<FailoverManager>,
    rate_limiter: Arc<RateLimiterRegistry>,
}

impl StateEngineService {
    pub fn new(
        engine: Arc<StateEngine>,
        failover: Arc<FailoverManager>,
        rate_limiter: Arc<RateLimiterRegistry>,
    ) -> Self {
        Self {
            engine,
            failover,
            rate_limiter,
        }
    }

    /// Create a tonic gRPC server from this service
    pub fn into_server(self) -> GrpcStateEngineServer<Self> {
        GrpcStateEngineServer::new(self)
    }
}

#[tonic::async_trait]
impl StateEngineRpc for StateEngineService {
    async fn append_event(
        &self,
        request: Request<proto::AppendEventRequest>,
    ) -> Result<Response<proto::AppendEventResponse>, Status> {
        let req = request.into_inner();

        // 租户级限流
        let limiter = self.rate_limiter.get_or_create(&req.tenant_id);
        if let Err(msg) = limiter.check().await {
            return Err(Status::resource_exhausted(msg));
        }

        // Validate payload size to prevent DoS
        if req.payload.len() > MAX_PAYLOAD_SIZE {
            warn!(
                "[gRPC] Payload too large: {} bytes (max: {})",
                req.payload.len(),
                MAX_PAYLOAD_SIZE
            );
            return Err(Status::invalid_argument(format!(
                "Payload too large: {} bytes (max: {})",
                req.payload.len(),
                MAX_PAYLOAD_SIZE
            )));
        }

        // Parse payload with proper error handling
        let payload = match serde_json::from_str(&req.payload) {
            Ok(p) => p,
            Err(e) => {
                warn!("[gRPC] Invalid JSON payload for event {}: {}", req.event_id, e);
                return Err(Status::invalid_argument(format!(
                    "Invalid JSON payload: {}",
                    e
                )));
            }
        };

        // 构造 StateEvent 并追加
        let event = crate::models::StateEvent {
            event_id: uuid::Uuid::parse_str(&req.event_id)
                .map_err(|e| Status::invalid_argument(format!("Invalid event_id: {}", e)))?,
            tenant_id: req.tenant_id,
            namespace: req.namespace,
            version: req.version,
            payload,
            timestamp: chrono::Utc::now().timestamp(),
        };

        match self.engine.append_event(event).await {
            Ok(_) => {
                info!("[gRPC] Event {} appended successfully", req.event_id);
                Ok(Response::new(proto::AppendEventResponse {
                    success: true,
                    message: "Event appended".to_string(),
                }))
            }
            Err(e) => {
                Err(Status::internal(format!("Failed to append event: {}", e)))
            }
        }
    }

    async fn get_events(
        &self,
        request: Request<proto::GetEventsRequest>,
    ) -> Result<Response<proto::GetEventsResponse>, Status> {
        let req = request.into_inner();

        match self.engine.get_events(&req.tenant_id, &req.namespace, req.from_version, None).await {
            Ok(events) => {
                let proto_events: Vec<proto::Event> = events
                    .iter()
                    .map(|e| proto::Event {
                        event_id: e.event_id.to_string(),
                        tenant_id: e.tenant_id.clone(),
                        namespace: e.namespace.clone(),
                        version: e.version,
                        payload: e.payload.to_string(),
                        timestamp: e.timestamp.to_string(),
                    })
                    .collect();

                let count = proto_events.len() as u64;
                Ok(Response::new(proto::GetEventsResponse {
                    events: proto_events,
                    count,
                }))
            }
            Err(e) => {
                Err(Status::internal(format!("Failed to get events: {}", e)))
            }
        }
    }

    async fn health_check(
        &self,
        _request: Request<proto::HealthCheckRequest>,
    ) -> Result<Response<proto::HealthCheckResponse>, Status> {
        // 检查 FailoverManager 各组件的健康状态
        let health_map = self.failover.health_status.read().await;
        let all_healthy = health_map.values().all(|s| *s == HealthStatus::Healthy);
        let status = if all_healthy { 0i32 } else { 2i32 };
        drop(health_map);

        Ok(Response::new(proto::HealthCheckResponse {
            status,
            request_count: 0, // TODO: 接入 MetricsRegistry 获取请求计数
            message: if status == 0 {
                "All components healthy".to_string()
            } else {
                "Some components degraded".to_string()
            },
        }))
    }
}
