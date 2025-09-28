# Kafka 测试指南

本指南介绍如何测试 clamber-web-core 的 Kafka 功能。

## 前置条件

### 1. 安装 Kafka

#### 使用 Docker Compose（推荐）

创建 `docker-compose.yml` 文件：

```yaml
version: '3.8'
services:
  zookeeper:
    image: confluentinc/cp-zookeeper:7.4.0
    hostname: zookeeper
    container_name: zookeeper
    ports:
      - "2181:2181"
    environment:
      ZOOKEEPER_CLIENT_PORT: 2181
      ZOOKEEPER_TICK_TIME: 2000

  kafka:
    image: confluentinc/cp-kafka:7.4.0
    hostname: kafka
    container_name: kafka
    depends_on:
      - zookeeper
    ports:
      - "9092:9092"
      - "9101:9101"
    environment:
      KAFKA_BROKER_ID: 1
      KAFKA_ZOOKEEPER_CONNECT: 'zookeeper:2181'
      KAFKA_LISTENER_SECURITY_PROTOCOL_MAP: PLAINTEXT:PLAINTEXT,PLAINTEXT_HOST:PLAINTEXT
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://kafka:29092,PLAINTEXT_HOST://localhost:9092
      KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1
      KAFKA_TRANSACTION_STATE_LOG_MIN_ISR: 1
      KAFKA_TRANSACTION_STATE_LOG_REPLICATION_FACTOR: 1
      KAFKA_GROUP_INITIAL_REBALANCE_DELAY_MS: 0
      KAFKA_JMX_PORT: 9101
      KAFKA_JMX_HOSTNAME: localhost

  kafka-ui:
    image: provectuslabs/kafka-ui:latest
    container_name: kafka-ui
    depends_on:
      - kafka
    ports:
      - "8080:8080"
    environment:
      KAFKA_CLUSTERS_0_NAME: local
      KAFKA_CLUSTERS_0_BOOTSTRAPSERVERS: kafka:29092
```

启动服务：

```bash
docker-compose up -d
```

#### 手动安装

1. 下载 Kafka：https://kafka.apache.org/downloads
2. 解压并启动 Zookeeper：
   ```bash
   bin/zookeeper-server-start.sh config/zookeeper.properties
   ```
3. 启动 Kafka：
   ```bash
   bin/kafka-server-start.sh config/server.properties
   ```

### 2. 创建测试主题

```bash
# 创建主题
docker exec kafka kafka-topics --create --topic test-topic --bootstrap-server localhost:9092 --partitions 3 --replication-factor 1

docker exec kafka kafka-topics --create --topic user-events --bootstrap-server localhost:9092 --partitions 3 --replication-factor 1

docker exec kafka kafka-topics --create --topic batch-topic --bootstrap-server localhost:9092 --partitions 3 --replication-factor 1

docker exec kafka kafka-topics --create --topic transaction-topic --bootstrap-server localhost:9092 --partitions 3 --replication-factor 1

# 查看主题列表
docker exec kafka kafka-topics --list --bootstrap-server localhost:9092
```

## 运行测试

### 1. 编译项目

```bash
cargo build
```

### 2. 运行 Kafka 示例

```bash
cargo run --example kafka_example
```

### 3. 运行特定测试

#### 生产者测试

```bash
# 创建生产者测试脚本
cat > test_producer.rs << 'EOF'
use clamber_web_core::kafka::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = KafkaProducerConfig::default();
    let producer = KafkaProducer::new(config)?;
    
    for i in 0..10 {
        let message = format!("Test message {}", i);
        producer.send_message("test-topic", Some(&format!("key_{}", i)), &message).await?;
        println!("发送消息: {}", message);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    producer.flush().await?;
    println!("所有消息发送完成");
    Ok(())
}
EOF

cargo run --bin test_producer
```

#### 消费者测试

```bash
# 创建消费者测试脚本
cat > test_consumer.rs << 'EOF'
use clamber_web_core::kafka::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = KafkaConsumerConfig::default();
    config.group_id = "test-consumer-group".to_string();
    config.auto_offset_reset = Some("earliest".to_string());
    
    let consumer = KafkaConsumer::new(config)?;
    consumer.subscribe(&["test-topic"])?;
    
    println!("开始消费消息...");
    for i in 0..10 {
        match consumer.consume_message_with_timeout(Duration::from_secs(5)).await? {
            Some(message) => {
                println!("收到消息 {}: {}", i + 1, String::from_utf8_lossy(message.payload().unwrap_or(&[])));
            }
            None => {
                println!("消息 {}: 超时", i + 1);
            }
        }
    }
    
    Ok(())
}
EOF

cargo run --bin test_consumer
```

## 验证结果

### 1. 使用 Kafka UI

访问 http://localhost:8080 查看：
- 主题列表
- 消息内容
- 消费者组状态
- 分区信息

### 2. 使用命令行工具

```bash
# 查看主题消息
docker exec kafka kafka-console-consumer --topic test-topic --bootstrap-server localhost:9092 --from-beginning

# 查看消费者组
docker exec kafka kafka-consumer-groups --bootstrap-server localhost:9092 --list

# 查看消费者组详情
docker exec kafka kafka-consumer-groups --bootstrap-server localhost:9092 --describe --group test-consumer-group
```

## 常见问题

### 1. 连接失败

**问题**：`ConnectionError: 连接失败`

**解决方案**：
- 确保 Kafka 服务正在运行
- 检查端口是否正确（默认 9092）
- 验证防火墙设置

### 2. 主题不存在

**问题**：`ConsumerError: 订阅主题失败`

**解决方案**：
- 确保主题已创建
- 检查主题名称拼写
- 验证主题权限

### 3. 消费者组问题

**问题**：消费者无法加入组

**解决方案**：
- 检查 `group.id` 配置
- 确保 `session.timeout.ms` 设置合理
- 重启消费者应用

### 4. 序列化错误

**问题**：`SerializationError: 序列化失败`

**解决方案**：
- 确保数据结构实现了 `Serialize` trait
- 检查 JSON 格式是否正确
- 验证字段类型匹配

## 性能测试

### 1. 生产者性能测试

```bash
# 使用 kafka-producer-perf-test
docker exec kafka kafka-producer-perf-test --topic test-topic --num-records 100000 --record-size 1024 --throughput 10000 --producer-props bootstrap.servers=localhost:9092
```

### 2. 消费者性能测试

```bash
# 使用 kafka-consumer-perf-test
docker exec kafka kafka-consumer-perf-test --topic test-topic --bootstrap-server localhost:9092 --messages 100000
```

## 监控和调试

### 1. 启用日志

在代码中添加：

```rust
tracing_subscriber::fmt::init();
```

### 2. 获取统计信息

```rust
// 生产者统计
let stats = producer.get_stats()?;
println!("生产者统计: {}", stats);

// 消费者统计
let stats = consumer.get_stats()?;
println!("消费者统计: {}", stats);
```

### 3. 健康检查

```rust
// 检查生产者连接
producer.flush().await?;

// 检查消费者订阅
let subscription = consumer.subscription()?;
println!("订阅的主题: {:?}", subscription);
```

## 最佳实践

1. **配置优化**：
   - 根据消息大小调整 `batch.size`
   - 根据延迟要求调整 `linger.ms`
   - 根据可靠性要求调整 `acks`

2. **错误处理**：
   - 实现重试机制
   - 记录错误日志
   - 监控失败率

3. **资源管理**：
   - 及时关闭生产者和消费者
   - 避免内存泄漏
   - 合理设置超时时间

4. **监控告警**：
   - 监控消息延迟
   - 监控消费滞后
   - 设置异常告警
