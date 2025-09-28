//! Kafka 使用示例
//!
//! 演示如何使用 clamber-web-core 的 Kafka 功能

use clamber_web_core::kafka::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
struct UserEvent {
    user_id: u64,
    event_type: String,
    timestamp: i64,
    data: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("Kafka 示例开始...");

    // 示例1: 基础生产者使用
    basic_producer_example().await?;

    // 示例2: 基础消费者使用
    basic_consumer_example().await?;

    // 示例3: 事务性生产者使用
    transactional_producer_example().await?;

    // 示例4: 高级消费者使用
    advanced_consumer_example().await?;

    // 示例5: 消费者组使用
    consumer_group_example().await?;

    println!("Kafka 示例完成！");
    Ok(())
}

/// 基础生产者示例
async fn basic_producer_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 基础生产者示例 ===");

    // 创建生产者配置
    let mut config = KafkaProducerConfig::default();
    config.base.bootstrap_servers = vec!["localhost:9092".to_string()];
    config.base.client_id = Some("example-producer".to_string());

    // 创建生产者
    let producer = KafkaProducer::new(config)?;
    println!("生产者创建成功");

    // 发送简单文本消息
    producer
        .send_message("test-topic", Some("key1"), "Hello Kafka!")
        .await?;
    println!("发送文本消息成功");

    // 发送序列化对象
    let user_event = UserEvent {
        user_id: 12345,
        event_type: "login".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        data: serde_json::json!({"ip": "192.168.1.1", "user_agent": "Mozilla/5.0"}),
    };

    producer
        .send_serialized("user-events", Some("user_12345"), &user_event)
        .await?;
    println!("发送序列化消息成功");

    // 批量发送消息
    let messages = vec![
        (Some("batch_key1".to_string()), b"Batch message 1".to_vec()),
        (Some("batch_key2".to_string()), b"Batch message 2".to_vec()),
        (Some("batch_key3".to_string()), b"Batch message 3".to_vec()),
    ];

    producer.send_batch("batch-topic", messages).await?;
    println!("批量发送消息成功");

    // 刷新缓冲区
    producer.flush().await?;
    println!("生产者缓冲区刷新完成");

    Ok(())
}

/// 基础消费者示例
async fn basic_consumer_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 基础消费者示例 ===");

    // 创建消费者配置
    let mut config = KafkaConsumerConfig::default();
    config.base.bootstrap_servers = vec!["localhost:9092".to_string()];
    config.group_id = "example-consumer-group".to_string();
    config.auto_offset_reset = Some("earliest".to_string());

    // 创建消费者
    let consumer = KafkaConsumer::new(config)?;
    println!("消费者创建成功");

    // 订阅主题
    consumer.subscribe(&["test-topic", "user-events"])?;
    println!("订阅主题成功");

    // 消费几条消息
    for i in 0..3 {
        match consumer
            .consume_message_with_timeout(Duration::from_secs(5))
            .await?
        {
            Some(message) => {
                println!(
                    "收到消息 {}: topic={}, partition={}, offset={}, key={:?}, payload={:?}",
                    i + 1,
                    message.topic(),
                    message.partition(),
                    message.offset(),
                    message.key(),
                    String::from_utf8_lossy(message.payload().unwrap_or(&[]))
                );
            }
            None => {
                println!("消息 {}: 超时，未收到消息", i + 1);
            }
        }
    }

    Ok(())
}

/// 事务性生产者示例
async fn transactional_producer_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 事务性生产者示例 ===");

    // 创建事务性生产者配置
    let mut config = KafkaProducerConfig::default();
    config.base.bootstrap_servers = vec!["localhost:9092".to_string()];
    config.acks = Some("all".to_string()); // 启用幂等性时必须设置为all
    config.enable_idempotence = Some(true);
    config.transactional_id = Some("example-transaction".to_string());

    // 创建事务性生产者
    let producer = TransactionalKafkaProducer::new(config, "example-transaction".to_string())?;
    println!("事务性生产者创建成功");

    // 初始化事务
    producer.init_transaction().await?;
    println!("事务初始化成功");

    // 开始事务
    producer.begin_transaction().await?;
    println!("事务开始");

    // 发送事务性消息
    producer
        .send_transactional_message(
            "transaction-topic",
            Some("tx_key1"),
            b"Transaction message 1",
        )
        .await?;
    producer
        .send_transactional_message(
            "transaction-topic",
            Some("tx_key2"),
            b"Transaction message 2",
        )
        .await?;
    println!("发送事务性消息成功");

    // 提交事务
    producer.commit_transaction().await?;
    println!("事务提交成功");

    Ok(())
}

/// 高级消费者示例
async fn advanced_consumer_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 高级消费者示例 ===");

    // 创建高级消费者配置
    let mut config = KafkaConsumerConfig::default();
    config.base.bootstrap_servers = vec!["localhost:9092".to_string()];
    config.group_id = "advanced-consumer-group".to_string();
    config.auto_offset_reset = Some("earliest".to_string());

    // 创建高级消费者
    let mut consumer = AdvancedKafkaConsumer::new(config)?;
    println!("高级消费者创建成功");

    // 注册消息处理函数
    consumer.register_handler("user-events".to_string(), |message| {
        if let Some(payload) = message.payload() {
            match serde_json::from_slice::<UserEvent>(payload) {
                Ok(user_event) => {
                    println!("处理用户事件: {:?}", user_event);
                }
                Err(e) => {
                    eprintln!("反序列化用户事件失败: {}", e);
                }
            }
        }
        Ok(())
    });

    consumer.register_handler("test-topic".to_string(), |message| {
        if let Some(payload) = message.payload() {
            println!("处理测试消息: {}", String::from_utf8_lossy(payload));
        }
        Ok(())
    });

    println!("消息处理函数注册成功");

    // 注意：在实际应用中，start_consuming 会无限循环
    // 这里只是演示，实际使用时应该在单独的线程中运行
    println!("高级消费者配置完成（实际使用时需要调用 start_consuming）");

    Ok(())
}

/// 消费者组示例
async fn consumer_group_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 消费者组示例 ===");

    // 创建消费者组配置
    let mut config = KafkaConsumerConfig::default();
    config.base.bootstrap_servers = vec!["localhost:9092".to_string()];
    config.group_id = "consumer-group-example".to_string();
    config.auto_offset_reset = Some("earliest".to_string());

    // 创建消费者组管理器
    let manager = ConsumerGroupManager::new(config, 3)?;
    println!(
        "消费者组管理器创建成功，包含 {} 个消费者",
        manager.consumer_count()
    );

    // 启动所有消费者
    manager
        .start_all(&["test-topic", "user-events", "batch-topic"])
        .await?;
    println!("所有消费者启动成功");

    // 获取第一个消费者并消费一条消息
    if let Some(consumer) = manager.get_consumer(0) {
        match consumer
            .consume_message_with_timeout(Duration::from_secs(3))
            .await?
        {
            Some(message) => {
                println!(
                    "消费者组中的消费者0收到消息: topic={}, payload={:?}",
                    message.topic(),
                    String::from_utf8_lossy(message.payload().unwrap_or(&[]))
                );
            }
            None => {
                println!("消费者组中的消费者0: 超时，未收到消息");
            }
        }
    }

    Ok(())
}

/// 配置示例
fn config_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 配置示例 ===");

    // 创建自定义生产者配置
    let mut producer_config = KafkaProducerConfig::default();
    producer_config.base.bootstrap_servers =
        vec!["kafka1:9092".to_string(), "kafka2:9092".to_string()];
    producer_config.base.client_id = Some("custom-producer".to_string());
    producer_config.acks = Some("-1".to_string()); // 等待所有副本确认
    producer_config.retries = Some(5);
    producer_config.compression_type = Some("gzip".to_string());
    producer_config.enable_idempotence = Some(true);

    println!("自定义生产者配置: {:?}", producer_config);

    // 创建自定义消费者配置
    let mut consumer_config = KafkaConsumerConfig::default();
    consumer_config.base.bootstrap_servers =
        vec!["kafka1:9092".to_string(), "kafka2:9092".to_string()];
    consumer_config.group_id = "custom-consumer-group".to_string();
    consumer_config.enable_auto_commit = Some(false); // 手动提交偏移量
    consumer_config.auto_offset_reset = Some("earliest".to_string());
    consumer_config.max_poll_records = Some(100);
    consumer_config.partition_assignment_strategy = Some("roundrobin".to_string());

    println!("自定义消费者配置: {:?}", consumer_config);

    // 序列化配置到文件
    let producer_yaml = serde_yaml::to_string(&producer_config)?;
    std::fs::write("producer_config.yaml", producer_yaml)?;
    println!("生产者配置已保存到 producer_config.yaml");

    let consumer_yaml = serde_yaml::to_string(&consumer_config)?;
    std::fs::write("consumer_config.yaml", consumer_yaml)?;
    println!("消费者配置已保存到 consumer_config.yaml");

    Ok(())
}

/// 错误处理示例
fn error_handling_example() {
    println!("\n=== 错误处理示例 ===");

    // 演示不同类型的错误
    let config_errors = vec![
        KafkaError::ConfigError("配置参数错误".to_string()),
        KafkaError::ProducerError("生产者创建失败".to_string()),
        KafkaError::ConsumerError("消费者订阅失败".to_string()),
        KafkaError::SendError("消息发送失败".to_string()),
        KafkaError::ReceiveError("消息接收失败".to_string()),
        KafkaError::SerializationError("序列化失败".to_string()),
        KafkaError::ConnectionError("连接失败".to_string()),
        KafkaError::TimeoutError("操作超时".to_string()),
    ];

    for error in config_errors {
        println!("错误类型: {}", error);
    }
}
