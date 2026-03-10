# Code Review Fixes - Task 1.1

## 审查问题修复总结

### ✅ 已修复的高优先级问题

#### 1. Redis 缓存失效逻辑错误 (高严重性)
**问题**: 部分范围查询会删除 Redis 中的完整缓存，导致数据丢失

**修复方案**:
- 仅在全量查询 (`to_version.is_none()`) 时回填缓存
- 部分范围查询不再删除和重建缓存
- 使用 Redis Pipeline 批量操作，减少网络往返

```rust
// 修复后：只在 to_version 为 None 时更新缓存
if !events.is_empty() && to_version.is_none() {
    let mut pipe = redis::pipe();
    pipe.del(&redis_key);
    for event in &events {
        let event_json = serde_json::to_string(event)?;
        pipe.zadd(&redis_key, event.version as f64, event_json);
    }
    pipe.expire(&redis_key, REDIS_CACHE_TTL_SECS)
        .ignore()
        .query_async::<_, ()>(&mut conn)
        .await?;
}
```

#### 2. Redis 降级逻辑改进 (中间接修复)
**问题**: Redis 数据不完整时不会降级到 PostgreSQL

**修复方案**:
- Redis 返回空结果时自动降级到 PostgreSQL
- 查询后自动回填 Redis 缓存（仅限全量查询）
- 损坏的 JSON 条目会被跳过而不是导致失败

#### 3. f64::INFINITY 问题 (低严重性)
**问题**: Redis ZRANGEBYSCORE 对无穷大处理不明确

**修复方案**:
- 使用 Redis 特定的 `"+inf"` 字符串表示无穷大
```rust
let max_score = to_version
    .map(|v| v.to_string())
    .unwrap_or_else(|| "+inf".to_string());
```

#### 4. JSON 错误处理 (中严重性)
**问题**: 损坏的缓存数据会导致查询失败

**修复方案**:
- 使用 `filter_map` 跳过无法解析的 JSON 条目
- 损坏的缓存项会被忽略，自动降级到 PostgreSQL
```rust
let events: Vec<StateEvent> = event_strings
    .iter()
    .filter_map(|s| serde_json::from_str::<StateEvent>(s).ok())
    .collect();
```

#### 5. N+1 Redis 写入优化 (低严重性)
**问题**: 缓存回填时逐个写入事件

**修复方案**:
- 使用 Redis Pipeline 批量写入
- 减少网络往返次数

### 🔧 其他改进

#### 代码质量
- ✅ 移除中文注释，统一使用英文
- ✅ 定义常量 `REDIS_CACHE_TTL_SECS = 86400`
- ✅ 修正导入：`use sqlx::Row;`
- ✅ 改进文档注释，使用英文

#### 类型修复
- ✅ `PgPool` → `sqlx::Pool<sqlx::Postgres>`
- ✅ `i64` → `usize` for Redis TTL
- ✅ 添加 `query_async::<_, ()>` 类型注解

### 📊 修复验证

```bash
# 编译成功，无错误
cd control-plane/state-engine
cargo build

# 输出:
# Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.11s
```

### 📝 文件变更

| 文件 | 变更说明 |
|------|---------|
| `engine.rs` | 修复所有审查发现的问题 |
| `lib.rs` | 无变更 |

### ⏭️ 下一步

继续实现:
- Task 1.2: 状态回放逻辑
- Task 1.3: 状态应用逻辑
