CREATE TABLE IF NOT EXISTS hot_events (
    event_id UUID PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,
    namespace VARCHAR(255) NOT NULL,
    version BIGINT NOT NULL,
    payload JSONB NOT NULL,
    timestamp BIGINT NOT NULL,
    CONSTRAINT unique_tenant_namespace_version UNIQUE (tenant_id, namespace, version)
);

CREATE INDEX IF NOT EXISTS idx_hot_events_tenant_namespace ON hot_events (tenant_id, namespace);

CREATE TABLE IF NOT EXISTS snapshots (
    snapshot_id UUID PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,
    namespace VARCHAR(255) NOT NULL,
    start_version BIGINT NOT NULL,
    end_version BIGINT NOT NULL,
    state_blob JSONB NOT NULL,
    created_at BIGINT NOT NULL,
    CONSTRAINT unique_snapshot_version UNIQUE (tenant_id, namespace, end_version)
);
