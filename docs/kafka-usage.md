# Kafka 使用指南

本文档介绍如何使用 clamber-web-core 的 Kafka 功能。

## 功能特性

- **生产者服务**：支持同步和异步消息发送
- **消费者服务**：支持单个和批量消息消费
- **事务支持**：支持事务性消息发送
- **配置管理**：支持自定义生产者和消费者配置
- **错误处理**：统一的错误处理机制
- **序列化支持**：内置 JSON 序列化支持

## 基本用法

### 1. 生产者使用

```rust
use clamber_web_core::kafka::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建生产者配置
    let mut config = KafkaProducerConfig::default();
    config.base.bootstrap_servers = vec!["localhost:9092".to_string()];
    
    // 创建生产者
    let producer = KafkaProducer::new(config)?;
    
    // 发送消息
    producer.send_message("test-topic", Some("key1"), "Hello Kafka!").await?;
    
    // 发送序列化对象
    #[derive(serde::Serialize)]
    struct UserEvent {
        user_id: u64,
        action: String,
    }
    
    let event = UserEvent {
        user_id: 123,
        action: "login".to_string(),
    };
    
    producer.send_serialized("user-events", Some("user_123"), &event).await?;
    
    Ok(())
}
```

### 2. 消费者使用

```rust
use clamber_web_core::kafka::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建消费者配置
    let mut config = KafkaConsumerConfig::default();
    config.base.bootstrap_servers = vec!["localhost:9092".to_string()];
    config.group_id = "my-consumer-group".to_string();
    config.auto_offset_reset = Some("earliest".to_string());
    
    // 创建消费者
    let consumer = KafkaConsumer::new(config)?;
    
    // 订阅主题
    consumer.subscribe(&["test-topic", "user-events"])?;
    
    // 消费消息
    loop {
        match consumer.consume_message_with_timeout(Duration::from_secs(5)).await? {
            Some(message) => {
                println!("收到消息: topic={}, payload={:?}",
                    message.topic(),
                    String::from_utf8_lossy(message.payload().unwrap_or(&[]))
                );
            }
            None => {
                println!("超时，未收到消息");
                break;
            }
        }
    }
    
    Ok(())
}
```

### 3. 事务性生产者

```rust
use clamber_web_core::kafka::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建事务性生产者配置
    let mut config = KafkaProducerConfig::default();
    config.base.bootstrap_servers = vec!["localhost:9092".to_string()];
    config.enable_idempotence = Some(true);
    
    // 创建事务性生产者
    let producer = TransactionalKafkaProducer::new(config, "my-transaction".to_string())?;
    
    // 初始化事务
    producer.init_transaction().await?;
    
    // 开始事务
    producer.begin_transaction().await?;
    
    // 发送事务性消息
    producer.send_transactional_message("transaction-topic", Some("key1"), b"Message 1").await?;
    producer.send_transactional_message("transaction-topic", Some("key2"), b"Message 2").await?;
    
    // 提交事务
    producer.commit_transaction().await?;
    
    Ok(())
}
```

## 配置选项

### 生产者配置

```rust
let mut config = KafkaProducerConfig::default();

// 基础配置
config.base.bootstrap_servers = vec!["kafka1:9092".to_string(), "kafka2:9092".to_string()];
config.base.client_id = Some("my-producer".to_string());

// 性能配置
config.acks = Some("-1".to_string()); // 等待所有副本确认
config.retries = Some(5);
config.batch_size = Some(32768);
config.linger_ms = Some(5);
config.compression_type = Some("gzip".to_string());

// 可靠性配置
config.enable_idempotence = Some(true);
```

### 消费者配置

```rust
let mut config = KafkaConsumerConfig::default();

// 基础配置
config.base.bootstrap_servers = vec!["kafka1:9092".to_string(), "kafka2:9092".to_string()];
config.group_id = "my-consumer-group".to_string();

// 消费配置
config.enable_auto_commit = Some(false); // 手动提交偏移量
config.auto_offset_reset = Some("earliest".to_string());
config.max_poll_records = Some(100);
config.session_timeout_ms = Some(30000);
```

## 高级用法

### 1. 使用构建器模式

```rust
let producer = KafkaClientBuilder::new()
    .with_producer_config(producer_config)
    .build_producer()?;

let consumer = KafkaClientBuilder::new()
    .with_consumer_config(consumer_config)
    .build_consumer()?;
```

### 2. 从配置文件创建

```rust
let producer = create_producer_from_config("producer_config.yaml")?;
let consumer = create_consumer_from_config("consumer_config.yaml")?;
```

### 3. 批量消费

```rust
let messages = consumer.consume_batch(10).await?;
for message in messages {
    println!("处理消息: {:?}", String::from_utf8_lossy(message.payload().unwrap_or(&[])));
}
```

### 4. 高级消费者

```rust
let mut consumer = AdvancedKafkaConsumer::new(config)?;

// 注册消息处理函数
consumer.register_handler("user-events".to_string(), |message| {
    println!("处理用户事件: {:?}", message);
    Ok(())
});

consumer.register_handler("system-events".to_string(), |message| {
    println!("处理系统事件: {:?}", message);
    Ok(())
});

// 开始消费（这会无限循环）
consumer.start_consuming(&["user-events", "system-events"]).await?;
```

## 错误处理

```rust
match producer.send_message("topic", Some("key"), "message").await {
    Ok(_) => println!("消息发送成功"),
    Err(KafkaError::ProducerError(e)) => println!("生产者错误: {}", e),
    Err(KafkaError::ConnectionError(e)) => println!("连接错误: {}", e),
    Err(KafkaError::SerializationError(e)) => println!("序列化错误: {}", e),
    Err(e) => println!("其他错误: {}", e),
}
```

## 最佳实践

1. **资源管理**：确保正确关闭生产者和消费者
2. **错误处理**：实现适当的重试和错误恢复机制
3. **性能优化**：根据业务需求调整批量大小和超时设置
4. **监控**：监控消息延迟、吞吐量和错误率
5. **配置管理**：使用外部配置文件管理Kafka设置

## 注意事项

1. 某些功能（如统计信息）在当前版本中可能未完全实现
2. 事务性功能需要Kafka 0.11.0+版本支持
3. 在生产环境中使用时，请确保正确配置安全认证
4. 消息提交功能可能需要根据具体的rdkafka版本进行调整
