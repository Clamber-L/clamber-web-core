# Axum + Kafka 集成指南

本指南介绍如何在 axum 项目中使用 `clamber-web-core` 的 Kafka 功能。

## 功能特性

✅ **KafkaAppState**: 为 axum 项目设计的应用状态，包含 Kafka producer 和 consumer  
✅ **轮询消费者**: 支持轮询接收消息的 consumer 功能  
✅ **配置管理**: 支持默认配置和配置文件两种方式  
✅ **错误处理**: 完善的错误处理和重试机制  
✅ **类型安全**: 完全类型安全的 API 设计  

## 快速开始

### 1. 基本使用

```rust
use axum::{extract::State, response::Json, routing::post, Router};
use clamber_web_core::kafka::*;
use std::sync::Arc;

type AppState = Arc<KafkaAppState>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 Kafka AppState
    let kafka_state = create_default_kafka_app_state(
        vec!["localhost:9092".to_string()],
        "my-consumer-group".to_string(),
    ).await?;

    // 启动轮询消费者
    start_polling_consumer(kafka_state.clone()).await;

    // 创建路由
    let app = Router::new()
        .route("/send-message", post(send_message))
        .with_state(Arc::new(kafka_state));

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn send_message(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    match state.send_message("my-topic", None, &payload.to_string()).await {
        Ok(_) => Json(serde_json::json!({"success": true})),
        Err(e) => Json(serde_json::json!({"error": e.to_string()})),
    }
}

async fn start_polling_consumer(state: AppState) {
    let topics = vec!["my-topic".to_string()];
    let polling_service = PollingConsumerService::new(
        (*state).clone(),
        topics,
        std::time::Duration::from_secs(1),
        10,
    );

    tokio::task::spawn(async move {
        let message_handler = |message: OwnedMessage| -> KafkaResult<()> {
            println!("收到消息: {:?}", String::from_utf8_lossy(
                message.payload().unwrap_or(&[])
            ));
            Ok(())
        };

        polling_service.start_polling(message_handler).await.unwrap();
    });
}
```

### 2. 使用配置文件

创建 `producer_config.yaml`:
```yaml
base:
  bootstrap_servers:
    - "localhost:9092"
  client_id: "my-producer"
acks: "1"
retries: 3
```

创建 `consumer_config.yaml`:
```yaml
base:
  bootstrap_servers:
    - "localhost:9092"
  client_id: "my-consumer"
group_id: "my-consumer-group"
enable_auto_commit: true
```

使用配置文件:
```rust
let kafka_state = create_kafka_app_state_from_config(
    "producer_config.yaml",
    "consumer_config.yaml",
).await?;
```

## 主要 API

### KafkaAppState

```rust
pub struct KafkaAppState {
    pub producer: Arc<KafkaProducer>,           // Kafka 生产者
    pub consumer: Arc<RwLock<KafkaConsumer>>,   // Kafka 消费者
    pub consumer_config: KafkaConsumerConfig,   // 消费者配置
}
```

**主要方法:**
- `send_message(topic, key, payload)` - 发送文本消息
- `send_serialized(topic, key, data)` - 发送序列化对象
- `poll_message(timeout)` - 轮询接收消息（带超时）
- `poll_batch(max_messages)` - 批量轮询消息
- `subscribe(topics)` - 订阅主题

### PollingConsumerService

```rust
pub struct PollingConsumerService {
    app_state: KafkaAppState,
    topics: Vec<String>,
    poll_interval: Duration,
    max_messages_per_poll: usize,
}
```

**主要方法:**
- `start_polling(handler)` - 开始轮询消费
- `start_polling_with_timeout(handler, timeout)` - 开始轮询消费（带超时控制）

## 示例项目

运行示例项目:

```bash
# 基本示例
cargo run --example axum_kafka_example

# 配置文件示例
cargo run --example axum_kafka_config_example
```

示例项目提供了完整的 HTTP API:
- `GET /` - 根路径信息
- `GET /health` - 健康检查
- `POST /send-message` - 发送消息
- `POST /send-user-message` - 发送用户消息
- `GET /producer-stats` - 获取生产者统计
- `GET /consumer-stats` - 获取消费者统计

## 测试 API

启动示例后，可以使用以下命令测试:

```bash
# 发送消息
curl -X POST http://localhost:3000/send-message \
  -H "Content-Type: application/json" \
  -d '{"topic": "test-topic", "key": "test-key", "message": "Hello Kafka!"}'

# 发送用户消息
curl -X POST http://localhost:3000/send-user-message \
  -H "Content-Type: application/json" \
  -d '{"topic": "user-messages", "user_id": 12345, "message": "Hello from user!"}'

# 健康检查
curl http://localhost:3000/health
```

## 配置选项

### 生产者配置

```yaml
base:
  bootstrap_servers: ["localhost:9092"]
  client_id: "my-producer"
  security_protocol: "PLAINTEXT"
  connection_timeout_ms: 30000
  request_timeout_ms: 30000

# 生产者特定配置
acks: "1"                    # 确认模式
retries: 3                   # 重试次数
retry_backoff_ms: 100        # 重试间隔
batch_size: 16384            # 批量大小
linger_ms: 0                 # 批量延迟
compression_type: "none"     # 压缩类型
enable_idempotence: false    # 幂等性
```

### 消费者配置

```yaml
base:
  bootstrap_servers: ["localhost:9092"]
  client_id: "my-consumer"
  security_protocol: "PLAINTEXT"
  connection_timeout_ms: 30000
  request_timeout_ms: 30000

# 消费者特定配置
group_id: "my-consumer-group"
enable_auto_commit: true
auto_commit_interval_ms: 5000
session_timeout_ms: 30000
heartbeat_interval_ms: 3000
auto_offset_reset: "latest"
partition_assignment_strategy: "range"
```

## 最佳实践

1. **资源管理**: 使用 `Arc<KafkaAppState>` 在多个处理器之间共享状态
2. **轮询配置**: 根据消息量调整轮询间隔和批量大小
3. **错误处理**: 实现重试机制和详细的错误日志
4. **监控**: 定期检查生产者和消费者统计信息
5. **健康检查**: 提供健康检查端点监控 Kafka 连接状态

## 故障排除

### 常见问题

1. **连接失败**: 检查 Kafka 服务器地址和网络连接
2. **消息发送失败**: 检查主题是否存在和权限设置
3. **消息消费失败**: 检查消费者组配置和主题订阅

### 调试技巧

- 启用详细日志记录
- 使用 Kafka 管理工具检查主题和分区
- 监控网络连接和资源使用情况

## 更多信息

详细的使用文档请参考: `docs/axum-kafka-integration.md`
