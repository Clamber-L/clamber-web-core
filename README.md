# Clamber Web Core

基于 clamber-core 构建的 Web 基础 crate，提供数据库、Redis、Kafka 等功能的统一接口。

## 特性

- 🗄️ **数据库管理**：基于 SeaORM 的数据库操作
- 🔴 **Redis 支持**：连接池管理和缓存操作
- 📨 **Kafka 消息队列**：生产者和消费者支持
- 🌐 **Web 框架集成**：基于 Axum 的 Web 服务
- ⚡ **按需引入**：支持 feature flags 选择性启用功能

## 快速开始

### 使用所有功能（默认）

```toml
[dependencies]
clamber-web-core = "0.1.1"
```

### 按需引入功能

```toml
[dependencies]
# 只使用数据库功能
clamber-web-core = { version = "0.1.1", default-features = false, features = ["database"] }

# 使用数据库和Redis
clamber-web-core = { version = "0.1.1", default-features = false, features = ["database", "redis"] }

# 使用Kafka消息队列
clamber-web-core = { version = "0.1.1", default-features = false, features = ["kafka"] }
```

## 可用的 Features

- `database`: 启用数据库模块（SeaORM）
- `redis`: 启用Redis模块
- `kafka`: 启用Kafka模块
- `full`: 启用所有功能
- `default`: 默认启用所有功能

## 使用示例

### 数据库操作

```rust
use clamber_web_core::DatabaseManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_manager = DatabaseManager::new("mysql://user:pass@localhost/db").await?;
    // 使用数据库功能...
    Ok(())
}
```

### Redis 操作

```rust
use clamber_web_core::RedisManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let redis_manager = RedisManager::new("redis://localhost:6379").await?;
    // 使用Redis功能...
    Ok(())
}
```

### Kafka 消息队列

```rust
use clamber_web_core::{KafkaProducer, KafkaConsumer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let producer = KafkaProducer::new(KafkaProducerConfig::default())?;
    let consumer = KafkaConsumer::new(KafkaConsumerConfig::default())?;
    // 使用Kafka功能...
    Ok(())
}
```

## 文档

- [Feature Flags 使用指南](docs/features.md)
- [Kafka 使用指南](docs/kafka-usage.md)
- [Kafka 测试指南](KAFKA_TEST_GUIDE.md)

## 许可证

MIT OR Apache-2.0