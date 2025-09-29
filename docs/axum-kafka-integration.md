# Axum + Kafka 集成指南

本指南介绍如何在 axum 项目中使用 `clamber-web-core` 的 Kafka 功能。

## 概述

`clamber-web-core` 提供了专门为 axum 项目设计的 Kafka 集成模块，包括：

- `KafkaAppState`: 包含 Kafka producer 和 consumer 的应用状态
- `PollingConsumerService`: 轮询消费者服务
- 便捷的配置和初始化函数

## 快速开始

### 1. 添加依赖

在你的 `Cargo.toml` 中添加以下依赖：

```toml
[dependencies]
clamber-web-core = "0.1.2"
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
```

### 2. 基本使用

```rust
use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use clamber_web_core::kafka::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

#[derive(Debug, Serialize, Deserialize)]
struct MessageRequest {
    topic: String,
    key: Option<String>,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

type AppState = Arc<KafkaAppState>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 Kafka AppState
    let kafka_state = create_default_kafka_app_state(
        vec!["localhost:9092".to_string()],
        "my-consumer-group".to_string(),
    )
    .await?;

    // 启动轮询消费者
    start_polling_consumer(kafka_state.clone()).await;

    // 创建路由
    let app = Router::new()
        .route("/send-message", post(send_message))
        .with_state(kafka_state);

    // 启动服务器
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn send_message(
    State(state): State<AppState>,
    Json(payload): Json<MessageRequest>,
) -> Json<ApiResponse> {
    match state
        .send_message(&payload.topic, payload.key.as_deref(), &payload.message)
        .await
    {
        Ok(_) => Json(ApiResponse {
            success: true,
            message: "消息发送成功".to_string(),
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            message: format!("发送失败: {}", e),
        }),
    }
}

async fn start_polling_consumer(state: AppState) {
    let topics = vec!["my-topic".to_string()];
    let polling_service = PollingConsumerService::new(
        state,
        topics,
        Duration::from_secs(1),
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

## 详细功能

### KafkaAppState

`KafkaAppState` 是主要的应用状态结构，包含：

```rust
pub struct KafkaAppState {
    pub producer: Arc<KafkaProducer>,           // Kafka 生产者
    pub consumer: Arc<RwLock<KafkaConsumer>>,   // Kafka 消费者
    pub consumer_config: KafkaConsumerConfig,   // 消费者配置
}
```

#### 主要方法

- `send_message()`: 发送文本消息
- `send_serialized()`: 发送序列化对象
- `poll_message()`: 轮询接收消息（带超时）
- `poll_message_blocking()`: 轮询接收消息（阻塞式）
- `poll_batch()`: 批量轮询消息
- `subscribe()`: 订阅主题
- `recreate_consumer()`: 重新创建消费者

### PollingConsumerService

`PollingConsumerService` 提供轮询消费功能：

```rust
pub struct PollingConsumerService {
    app_state: KafkaAppState,
    topics: Vec<String>,
    poll_interval: Duration,
    max_messages_per_poll: usize,
}
```

#### 主要方法

- `start_polling()`: 开始轮询消费
- `start_polling_with_timeout()`: 开始轮询消费（带超时控制）

## 配置

### 使用默认配置

```rust
let kafka_state = create_default_kafka_app_state(
    vec!["localhost:9092".to_string()],
    "my-consumer-group".to_string(),
).await?;
```

### 使用配置文件

创建生产者配置文件 `producer_config.yaml`：

```yaml
base:
  bootstrap_servers:
    - "localhost:9092"
  client_id: "my-producer"
  security_protocol: "PLAINTEXT"
  connection_timeout_ms: 30000
  request_timeout_ms: 30000

acks: "1"
retries: 3
retry_backoff_ms: 100
batch_size: 16384
compression_type: "none"
enable_idempotence: false
```

创建消费者配置文件 `consumer_config.yaml`：

```yaml
base:
  bootstrap_servers:
    - "localhost:9092"
  client_id: "my-consumer"
  security_protocol: "PLAINTEXT"
  connection_timeout_ms: 30000
  request_timeout_ms: 30000

group_id: "my-consumer-group"
enable_auto_commit: true
auto_commit_interval_ms: 5000
session_timeout_ms: 30000
auto_offset_reset: "latest"
```

使用配置文件创建 AppState：

```rust
let kafka_state = create_kafka_app_state_from_config(
    "producer_config.yaml",
    "consumer_config.yaml",
).await?;
```

## 高级用法

### 自定义消息处理

```rust
async fn start_custom_consumer(state: AppState) {
    let topics = vec!["user-events".to_string(), "notifications".to_string()];
    let polling_service = PollingConsumerService::new(
        state,
        topics,
        Duration::from_secs(1),
        10,
    );

    tokio::task::spawn(async move {
        let message_handler = |message: OwnedMessage| -> KafkaResult<()> {
            let topic = message.topic();
            let payload = message.payload().unwrap_or(&[]);
            
            match topic {
                "user-events" => {
                    // 处理用户事件
                    if let Ok(user_event) = serde_json::from_slice::<UserEvent>(payload) {
                        handle_user_event(user_event).await;
                    }
                }
                "notifications" => {
                    // 处理通知
                    let notification = String::from_utf8_lossy(payload);
                    send_notification(&notification).await;
                }
                _ => {
                    println!("未知主题: {}", topic);
                }
            }
            
            Ok(())
        };

        polling_service.start_polling(message_handler).await.unwrap();
    });
}
```

### 错误处理

```rust
async fn send_message_with_retry(
    state: &AppState,
    topic: &str,
    key: Option<&str>,
    message: &str,
    max_retries: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    for attempt in 1..=max_retries {
        match state.send_message(topic, key, message).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                if attempt == max_retries {
                    return Err(e.into());
                }
                println!("发送失败，重试 {}/{}: {}", attempt, max_retries, e);
                tokio::time::sleep(Duration::from_millis(1000 * attempt)).await;
            }
        }
    }
    unreachable!()
}
```

### 健康检查

```rust
async fn health_check(State(state): State<AppState>) -> Json<ApiResponse> {
    // 检查生产者状态
    let producer_stats = state.get_producer_stats();
    
    // 检查消费者状态
    let consumer_stats = state.get_consumer_stats().await;
    
    let is_healthy = producer_stats.is_ok() && consumer_stats.is_ok();
    
    Json(ApiResponse {
        success: is_healthy,
        message: if is_healthy {
            "Kafka 连接正常".to_string()
        } else {
            "Kafka 连接异常".to_string()
        },
    })
}
```

## 最佳实践

### 1. 资源管理

- 使用 `Arc<KafkaAppState>` 在多个处理器之间共享状态
- 合理设置轮询间隔和批量大小
- 及时处理错误和异常情况

### 2. 性能优化

- 根据消息量调整 `max_messages_per_poll`
- 使用批量发送减少网络开销
- 合理配置 Kafka 客户端的缓冲区大小

### 3. 错误处理

- 实现重试机制
- 记录详细的错误日志
- 提供健康检查端点

### 4. 监控

- 定期检查生产者和消费者统计信息
- 监控消息处理延迟
- 设置适当的告警机制

## 示例项目

完整的使用示例请参考：

- `examples/axum_kafka_example.rs`: 基本使用示例
- `examples/axum_kafka_config_example.rs`: 配置文件使用示例

## 故障排除

### 常见问题

1. **连接失败**
   - 检查 Kafka 服务器地址和端口
   - 确认网络连接正常
   - 检查防火墙设置

2. **消息发送失败**
   - 检查主题是否存在
   - 确认生产者配置正确
   - 检查权限设置

3. **消息消费失败**
   - 检查消费者组配置
   - 确认主题订阅正确
   - 检查偏移量设置

### 调试技巧

- 启用详细日志记录
- 使用 Kafka 管理工具检查主题和分区
- 监控网络连接和资源使用情况
