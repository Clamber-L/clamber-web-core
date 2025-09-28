# Feature Flags 使用指南

clamber-web-core 支持按需引入功能模块，就像 tokio 那样。你可以选择性地启用需要的功能，减少依赖和编译时间。

## 可用的 Features

- `database`: 启用数据库模块（SeaORM）
- `redis`: 启用Redis模块
- `kafka`: 启用Kafka模块
- `full`: 启用所有功能
- `default`: 默认启用所有功能

## 使用方式

### 1. 使用默认功能（所有模块）

```toml
[dependencies]
clamber-web-core = "0.1.1"
```

### 2. 只启用数据库功能

```toml
[dependencies]
clamber-web-core = { version = "0.1.1", default-features = false, features = ["database"] }
```

### 3. 只启用Redis功能

```toml
[dependencies]
clamber-web-core = { version = "0.1.1", default-features = false, features = ["redis"] }
```

### 4. 只启用Kafka功能

```toml
[dependencies]
clamber-web-core = { version = "0.1.1", default-features = false, features = ["kafka"] }
```

### 5. 组合使用

```toml
[dependencies]
clamber-web-core = { version = "0.1.1", default-features = false, features = ["database", "redis"] }
```

### 6. 启用所有功能

```toml
[dependencies]
clamber-web-core = { version = "0.1.1", features = ["full"] }
```

## 代码中的使用

根据启用的features，相应的模块和类型会被导出：

```rust
// 如果启用了 database feature
use clamber_web_core::DatabaseManager;

// 如果启用了 redis feature  
use clamber_web_core::RedisManager;

// 如果启用了 kafka feature
use clamber_web_core::{KafkaProducer, KafkaConsumer};
```

## 条件编译

你也可以在代码中使用条件编译：

```rust
#[cfg(feature = "database")]
use clamber_web_core::DatabaseManager;

#[cfg(feature = "redis")]
use clamber_web_core::RedisManager;

#[cfg(feature = "kafka")]
use clamber_web_core::{KafkaProducer, KafkaConsumer};
```

## 依赖关系

- `database` feature 依赖：`sea-orm`, `clamber-core`
- `redis` feature 依赖：`redis`, `clamber-core`
- `kafka` feature 依赖：`rdkafka`

## 性能优势

通过选择性启用features，你可以：

1. **减少编译时间**：只编译需要的模块
2. **减少依赖**：避免引入不需要的依赖
3. **减少二进制大小**：只包含使用的代码
4. **提高灵活性**：根据项目需求定制功能

## 示例项目

### 只使用数据库的Web应用

```toml
[dependencies]
clamber-web-core = { version = "0.1.1", default-features = false, features = ["database"] }
```

```rust
use clamber_web_core::DatabaseManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_manager = DatabaseManager::new("mysql://user:pass@localhost/db").await?;
    // 使用数据库功能...
    Ok(())
}
```

### 使用Redis缓存的Web应用

```toml
[dependencies]
clamber-web-core = { version = "0.1.1", default-features = false, features = ["database", "redis"] }
```

```rust
use clamber_web_core::{DatabaseManager, RedisManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_manager = DatabaseManager::new("mysql://user:pass@localhost/db").await?;
    let redis_manager = RedisManager::new("redis://localhost:6379").await?;
    // 使用数据库和Redis功能...
    Ok(())
}
```

### 消息队列应用

```toml
[dependencies]
clamber-web-core = { version = "0.1.1", default-features = false, features = ["kafka"] }
```

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
