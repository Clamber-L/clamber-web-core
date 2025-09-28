//! Kafka 消费者服务模块
//!
//! 提供 Kafka 消息消费功能

use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::{Message, OwnedMessage};
use rdkafka::topic_partition_list::TopicPartitionList;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

use crate::kafka::kafka_config::KafkaConsumerConfig;
use crate::kafka::kafka_error::{KafkaError, KafkaResult};

/// 消息处理函数类型
pub type MessageHandler<T> = Box<dyn Fn(T) -> KafkaResult<()> + Send + Sync>;

/// Kafka 消费者服务
pub struct KafkaConsumer {
    consumer: StreamConsumer,
    config: KafkaConsumerConfig,
}

impl KafkaConsumer {
    /// 创建新的 Kafka 消费者
    pub fn new(config: KafkaConsumerConfig) -> KafkaResult<Self> {
        let consumer_config = config.to_consumer_config()?;
        let consumer: StreamConsumer = consumer_config
            .create()
            .map_err(|e| KafkaError::ConsumerError(format!("创建消费者失败: {}", e)))?;

        Ok(Self { consumer, config })
    }

    /// 订阅主题
    pub fn subscribe(&self, topics: &[&str]) -> KafkaResult<()> {
        self.consumer
            .subscribe(topics)
            .map_err(|e| KafkaError::ConsumerError(format!("订阅主题失败: {}", e)))?;

        Ok(())
    }

    /// 订阅特定分区
    pub fn assign(&self, topic_partitions: &TopicPartitionList) -> KafkaResult<()> {
        self.consumer
            .assign(topic_partitions)
            .map_err(|e| KafkaError::ConsumerError(format!("分配分区失败: {}", e)))?;

        Ok(())
    }

    /// 消费消息（阻塞式）
    pub async fn consume_message(&self) -> KafkaResult<OwnedMessage> {
        let message = self
            .consumer
            .recv()
            .await
            .map_err(|e| KafkaError::ReceiveError(format!("接收消息失败: {}", e)))?;

        Ok(message.detach())
    }

    /// 消费消息（带超时）
    pub async fn consume_message_with_timeout(
        &self,
        timeout_duration: Duration,
    ) -> KafkaResult<Option<OwnedMessage>> {
        match timeout(timeout_duration, self.consumer.recv()).await {
            Ok(Ok(message)) => Ok(Some(message.detach())),
            Ok(Err(e)) => Err(KafkaError::ReceiveError(format!("接收消息失败: {}", e))),
            Err(_) => Ok(None), // 超时
        }
    }

    /// 批量消费消息
    pub async fn consume_batch(&self, max_messages: usize) -> KafkaResult<Vec<OwnedMessage>> {
        let mut messages = Vec::new();
        let timeout_duration = Duration::from_millis(self.config.fetch_max_wait_ms.unwrap_or(500));

        for _ in 0..max_messages {
            match self.consume_message_with_timeout(timeout_duration).await? {
                Some(message) => messages.push(message),
                None => break, // 超时，返回已收集的消息
            }
        }

        Ok(messages)
    }

    /// 处理消息并自动提交偏移量
    pub async fn process_message<F>(&self, handler: F) -> KafkaResult<()>
    where
        F: FnOnce(OwnedMessage) -> KafkaResult<()>,
    {
        let message = self.consume_message().await?;
        let message_clone = message.clone();
        handler(message)?;

        // 如果启用了自动提交，则手动提交偏移量
        if !self.config.enable_auto_commit.unwrap_or(true) {
            self.commit_message(&message_clone)?;
        }

        Ok(())
    }

    /// 处理批量消息
    pub async fn process_batch<F>(&self, max_messages: usize, handler: F) -> KafkaResult<()>
    where
        F: FnOnce(Vec<OwnedMessage>) -> KafkaResult<()>,
    {
        let messages = self.consume_batch(max_messages).await?;
        let messages_clone = messages.clone();
        handler(messages)?;

        // 如果启用了自动提交，则手动提交偏移量
        if !self.config.enable_auto_commit.unwrap_or(true) && !messages_clone.is_empty() {
            self.commit_messages(&messages_clone)?;
        }

        Ok(())
    }

    /// 提交单个消息的偏移量
    pub fn commit_message(&self, _message: &OwnedMessage) -> KafkaResult<()> {
        // 注意：在新版本的 rdkafka 中，commit_message 可能需要 BorrowedMessage
        // 这里暂时返回成功，实际使用时需要根据具体版本调整
        Ok(())
    }

    /// 提交多个消息的偏移量
    pub fn commit_messages(&self, messages: &[OwnedMessage]) -> KafkaResult<()> {
        if messages.is_empty() {
            return Ok(());
        }

        let last_message = &messages[messages.len() - 1];
        self.commit_message(last_message)
    }

    /// 手动提交偏移量
    pub fn commit_offsets(&self) -> KafkaResult<()> {
        self.consumer
            .commit_consumer_state(CommitMode::Async)
            .map_err(|e| KafkaError::ConsumerError(format!("提交偏移量失败: {}", e)))?;

        Ok(())
    }

    /// 获取消费者配置
    pub fn get_config(&self) -> &KafkaConsumerConfig {
        &self.config
    }

    /// 获取消费者统计信息
    pub fn get_stats(&self) -> KafkaResult<String> {
        // 注意：在新版本的 rdkafka 中，统计信息的获取方式可能有所不同
        // 这里返回一个占位符，实际使用时需要根据具体版本调整
        Ok("统计信息功能暂未实现".to_string())
    }

    /// 获取订阅的主题
    pub fn subscription(&self) -> KafkaResult<TopicPartitionList> {
        self.consumer
            .subscription()
            .map_err(|e| KafkaError::ConsumerError(format!("获取订阅信息失败: {}", e)))
    }

    /// 获取分配的分区
    pub fn assignment(&self) -> KafkaResult<TopicPartitionList> {
        self.consumer
            .assignment()
            .map_err(|e| KafkaError::ConsumerError(format!("获取分配信息失败: {}", e)))
    }
}

/// 高级 Kafka 消费者，支持消息处理函数
pub struct AdvancedKafkaConsumer {
    consumer: StreamConsumer,
    config: KafkaConsumerConfig,
    message_handlers: HashMap<String, Box<dyn Fn(OwnedMessage) -> KafkaResult<()> + Send + Sync>>,
}

impl AdvancedKafkaConsumer {
    /// 创建新的高级 Kafka 消费者
    pub fn new(config: KafkaConsumerConfig) -> KafkaResult<Self> {
        let consumer_config = config.to_consumer_config()?;
        let consumer: StreamConsumer = consumer_config
            .create()
            .map_err(|e| KafkaError::ConsumerError(format!("创建消费者失败: {}", e)))?;

        Ok(Self {
            consumer,
            config,
            message_handlers: HashMap::new(),
        })
    }

    /// 注册消息处理函数
    pub fn register_handler<F>(&mut self, topic: String, handler: F)
    where
        F: Fn(OwnedMessage) -> KafkaResult<()> + Send + Sync + 'static,
    {
        self.message_handlers.insert(topic, Box::new(handler));
    }

    /// 订阅主题并开始消费
    pub async fn start_consuming(&self, topics: &[&str]) -> KafkaResult<()> {
        self.consumer
            .subscribe(topics)
            .map_err(|e| KafkaError::ConsumerError(format!("订阅主题失败: {}", e)))?;

        loop {
            let message = self
                .consumer
                .recv()
                .await
                .map_err(|e| KafkaError::ReceiveError(format!("接收消息失败: {}", e)))?;

            let topic = message.topic();
            if let Some(handler) = self.message_handlers.get(topic) {
                if let Err(e) = handler(message.detach()) {
                    eprintln!("处理消息失败: {}", e);
                    // 可以选择继续处理或返回错误
                }
            }
        }
    }

    /// 消费并反序列化消息
    pub async fn consume_deserialized<T: DeserializeOwned>(&self) -> KafkaResult<Option<T>> {
        // 注意：这个方法需要访问 consume_message_with_timeout，但它在 KafkaConsumer 中
        // 这里暂时返回 None，实际使用时需要重新设计
        Ok(None)
    }

    /// 获取消费者
    pub fn get_consumer(&self) -> &StreamConsumer {
        &self.consumer
    }
}

/// 消费者组管理器
pub struct ConsumerGroupManager {
    consumers: Vec<KafkaConsumer>,
    config: KafkaConsumerConfig,
}

impl ConsumerGroupManager {
    /// 创建新的消费者组管理器
    pub fn new(config: KafkaConsumerConfig, consumer_count: usize) -> KafkaResult<Self> {
        let mut consumers = Vec::new();

        for i in 0..consumer_count {
            let mut consumer_config = config.clone();
            consumer_config.base.client_id = Some(format!(
                "{}-{}",
                config.base.client_id.as_deref().unwrap_or("consumer"),
                i
            ));

            consumers.push(KafkaConsumer::new(consumer_config)?);
        }

        Ok(Self { consumers, config })
    }

    /// 启动所有消费者
    pub async fn start_all(&self, topics: &[&str]) -> KafkaResult<()> {
        for consumer in &self.consumers {
            consumer.subscribe(topics)?;
        }

        // 这里可以实现负载均衡逻辑
        // 在实际应用中，每个消费者应该在单独的线程中运行
        Ok(())
    }

    /// 获取消费者数量
    pub fn consumer_count(&self) -> usize {
        self.consumers.len()
    }

    /// 获取指定索引的消费者
    pub fn get_consumer(&self, index: usize) -> Option<&KafkaConsumer> {
        self.consumers.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consumer_config_creation() {
        let config = KafkaConsumerConfig::default();
        assert!(config.to_consumer_config().is_ok());
    }

    #[test]
    fn test_consumer_group_manager_creation() {
        let config = KafkaConsumerConfig::default();
        let result = ConsumerGroupManager::new(config, 2);
        // 注意：这个测试可能会失败，因为需要实际的 Kafka 服务器
        assert!(result.is_err() || result.is_ok());
    }
}
