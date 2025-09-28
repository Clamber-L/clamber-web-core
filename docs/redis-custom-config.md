# Redis 自定义配置使用指南

## 🎯 概述

现在您可以通过 `ConnectionManagerConfig` 来自定义 Redis 连接池的行为，包括超时设置、重试策略等。

## ⚙️ 可配置参数

### 基本配置

```rust
use clamber_web_core::redis::RedisConfig;

let config = RedisConfig {
    url: "redis://localhost:6379".to_string(),
    database_index: 0,
    // 自定义参数...
};
```

### 超时配置

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `connection_timeout_secs` | u64 | 30 | 连接建立超时时间（秒） |
| `response_timeout_secs` | u64 | 0 | 响应超时时间（秒），0表示无超时 |

### 重试配置

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `retry_count` | usize | 6 | 连接失败时的重试次数 |
| `retry_factor_ms` | u64 | 100 | 重试延迟因子（毫秒） |
| `max_retry_delay_ms` | u64 | 0 | 最大重试延迟（毫秒），0表示无限制 |

## 🚀 使用示例

### 示例1: 快速连接配置

```rust
use clamber_web_core::redis::{RedisConfig, RedisConnection};

// 快速连接，短超时
let fast_config = RedisConfig {
    url: "redis://localhost:6379".to_string(),
    database_index: 0,
    connection_timeout_secs: 5,   // 5秒连接超时
    response_timeout_secs: 2,     // 2秒响应超时
    retry_count: 3,               // 3次重试
    retry_factor_ms: 100,         // 100ms重试延迟
    max_retry_delay_ms: 1000,     // 最大1秒延迟
};

let mut connection = RedisConnection::new(fast_config).await?;
```

### 示例2: 稳定连接配置

```rust
// 稳定连接，长超时
let stable_config = RedisConfig {
    url: "redis://localhost:6379".to_string(),
    database_index: 0,
    connection_timeout_secs: 60,  // 60秒连接超时
    response_timeout_secs: 30,    // 30秒响应超时
    retry_count: 10,              // 10次重试
    retry_factor_ms: 500,         // 500ms重试延迟
    max_retry_delay_ms: 10000,    // 最大10秒延迟
};

let mut connection = RedisConnection::new(stable_config).await?;
```

### 示例3: 生产环境配置

```rust
// 生产环境推荐配置
let production_config = RedisConfig {
    url: std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
    database_index: 0,
    connection_timeout_secs: 30,  // 30秒连接超时
    response_timeout_secs: 10,    // 10秒响应超时
    retry_count: 5,               // 5次重试
    retry_factor_ms: 200,         // 200ms重试延迟
    max_retry_delay_ms: 5000,     // 最大5秒延迟
};

let mut connection = RedisConnection::new(production_config).await?;
```

## 🔧 配置策略

### 根据使用场景选择配置

#### 1. 高并发场景
```rust
let high_concurrency_config = RedisConfig {
    connection_timeout_secs: 10,  // 较短连接超时
    response_timeout_secs: 3,     // 较短响应超时
    retry_count: 3,               // 较少重试次数
    retry_factor_ms: 100,         // 较短重试延迟
    ..Default::default()
};
```

#### 2. 网络不稳定场景
```rust
let unstable_network_config = RedisConfig {
    connection_timeout_secs: 60,  // 较长连接超时
    response_timeout_secs: 30,    // 较长响应超时
    retry_count: 10,              // 较多重试次数
    retry_factor_ms: 1000,        // 较长重试延迟
    max_retry_delay_ms: 30000,    // 最大30秒延迟
    ..Default::default()
};
```

#### 3. 开发测试场景
```rust
let dev_config = RedisConfig {
    connection_timeout_secs: 5,   // 快速失败
    response_timeout_secs: 2,     // 快速响应
    retry_count: 1,               // 最少重试
    retry_factor_ms: 50,          // 快速重试
    ..Default::default()
};
```

## 📊 重试机制详解

### 重试延迟计算

重试延迟使用指数退避算法：

```
延迟 = min(重试因子 * (指数基数 ^ 重试次数), 最大延迟)
```

其中：
- 指数基数 = 2（固定）
- 重试因子 = `retry_factor_ms`
- 最大延迟 = `max_retry_delay_ms`

### 重试示例

假设配置：
- `retry_count = 3`
- `retry_factor_ms = 100`
- `max_retry_delay_ms = 1000`

重试时间表：
1. 第1次重试：100ms
2. 第2次重试：200ms  
3. 第3次重试：400ms

## 🎯 最佳实践

### 1. 超时设置建议

| 场景 | 连接超时 | 响应超时 | 说明 |
|------|----------|----------|------|
| 本地开发 | 5-10秒 | 2-5秒 | 快速失败，便于调试 |
| 测试环境 | 10-30秒 | 5-10秒 | 平衡速度和稳定性 |
| 生产环境 | 30-60秒 | 10-30秒 | 确保稳定性 |

### 2. 重试策略建议

| 场景 | 重试次数 | 重试因子 | 最大延迟 | 说明 |
|------|----------|----------|----------|------|
| 高可用服务 | 5-10次 | 200-500ms | 5-10秒 | 确保服务可用性 |
| 批处理任务 | 3-5次 | 100-200ms | 2-5秒 | 平衡重试和效率 |
| 实时应用 | 1-3次 | 50-100ms | 1-2秒 | 快速失败，避免延迟 |

### 3. 环境变量配置

```rust
use std::env;

let config = RedisConfig {
    url: env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
    database_index: env::var("REDIS_DB")
        .unwrap_or_else(|_| "0".to_string())
        .parse()
        .unwrap_or(0),
    connection_timeout_secs: env::var("REDIS_CONNECTION_TIMEOUT")
        .unwrap_or_else(|_| "30".to_string())
        .parse()
        .unwrap_or(30),
    response_timeout_secs: env::var("REDIS_RESPONSE_TIMEOUT")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or(10),
    retry_count: env::var("REDIS_RETRY_COUNT")
        .unwrap_or_else(|_| "5".to_string())
        .parse()
        .unwrap_or(5),
    retry_factor_ms: env::var("REDIS_RETRY_FACTOR_MS")
        .unwrap_or_else(|_| "200".to_string())
        .parse()
        .unwrap_or(200),
    max_retry_delay_ms: env::var("REDIS_MAX_RETRY_DELAY_MS")
        .unwrap_or_else(|_| "5000".to_string())
        .parse()
        .unwrap_or(5000),
};
```

## 🚨 注意事项

1. **超时设置**：过短的超时可能导致连接失败，过长的超时可能导致响应缓慢
2. **重试次数**：过多的重试可能增加系统负载，过少的重试可能降低可用性
3. **网络环境**：根据网络环境调整配置，内网环境可以使用更短的超时
4. **业务需求**：根据业务对延迟和可用性的要求选择合适的配置

## 📚 相关资源

- [Redis Crate ConnectionManagerConfig 文档](https://docs.rs/redis/0.32.5/redis/aio/struct.ConnectionManagerConfig.html)
- [连接池示例代码](../examples/redis_pool_example.rs)

---

通过自定义配置，您可以根据具体需求优化 Redis 连接的行为，在性能和稳定性之间找到最佳平衡点。
