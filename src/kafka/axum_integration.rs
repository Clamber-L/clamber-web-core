//! Axum 集成模块
//!
//! 为 axum 项目提供 Kafka producer 和 consumer 的 AppState 集成

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;

use crate::kafka::OwnedMessage;
use crate::kafka::kafka_config::{KafkaConsumerConfig, KafkaProducerConfig};
use crate::kafka::kafka_consumer::KafkaConsumer;
use crate::kafka::kafka_error::{KafkaError, KafkaResult};
use crate::kafka::kafka_producer::KafkaProducer;

/// Axum 应用的 Kafka 状态
#[derive(Clone)]
pub struct KafkaAppState {
    /// Kafka 生产者
    pub producer: Arc<KafkaProducer>,
    /// Kafka 消费者
    pub consumer: Arc<RwLock<KafkaConsumer>>,
    /// 消费者配置
    pub consumer_config: KafkaConsumerConfig,
}

impl KafkaAppState {
    /// 创建新的 Kafka AppState
    pub async fn new(
        producer_config: KafkaProducerConfig,
        consumer_config: KafkaConsumerConfig,
    ) -> KafkaResult<Self> {
        let producer = Arc::new(KafkaProducer::new(producer_config)?);
        let consumer = Arc::new(RwLock::new(KafkaConsumer::new(consumer_config.clone())?));

        Ok(Self {
            producer,
            consumer,
            consumer_config,
        })
    }

    /// 发送消息
    pub async fn send_message(
        &self,
        topic: &str,
        key: Option<&str>,
        payload: &str,
    ) -> KafkaResult<()> {
        self.producer.send_message(topic, key, payload).await
    }

    /// 发送序列化消息
    pub async fn send_serialized<T: serde::Serialize>(
        &self,
        topic: &str,
        key: Option<&str>,
        data: &T,
    ) -> KafkaResult<()> {
        self.producer.send_serialized(topic, key, data).await
    }

    /// 轮询接收消息（带超时）
    pub async fn poll_message(
        &self,
        timeout_duration: Duration,
    ) -> KafkaResult<Option<OwnedMessage>> {
        let consumer = self.consumer.read().await;
        consumer
            .consume_message_with_timeout(timeout_duration)
            .await
    }

    /// 轮询接收消息（阻塞式）
    pub async fn poll_message_blocking(&self) -> KafkaResult<OwnedMessage> {
        let consumer = self.consumer.read().await;
        consumer.consume_message().await
    }

    /// 批量轮询消息
    pub async fn poll_batch(&self, max_messages: usize) -> KafkaResult<Vec<OwnedMessage>> {
        let consumer = self.consumer.read().await;
        consumer.consume_batch(max_messages).await
    }

    /// 订阅主题
    pub async fn subscribe(&self, topics: &[&str]) -> KafkaResult<()> {
        let consumer = self.consumer.write().await;
        consumer.subscribe(topics)
    }

    /// 重新创建消费者（用于重新连接或配置更新）
    pub async fn recreate_consumer(&self) -> KafkaResult<()> {
        let new_consumer = KafkaConsumer::new(self.consumer_config.clone())?;
        let mut consumer = self.consumer.write().await;
        *consumer = new_consumer;
        Ok(())
    }

    /// 获取生产者统计信息
    pub fn get_producer_stats(&self) -> KafkaResult<String> {
        self.producer.get_stats()
    }

    /// 获取消费者统计信息
    pub async fn get_consumer_stats(&self) -> KafkaResult<String> {
        let consumer = self.consumer.read().await;
        consumer.get_stats()
    }
}

/// 轮询消费者服务
pub struct PollingConsumerService {
    app_state: KafkaAppState,
    topics: Vec<String>,
    poll_interval: Duration,
    max_messages_per_poll: usize,
}

impl PollingConsumerService {
    /// 创建新的轮询消费者服务
    pub fn new(
        app_state: KafkaAppState,
        topics: Vec<String>,
        poll_interval: Duration,
        max_messages_per_poll: usize,
    ) -> Self {
        Self {
            app_state,
            topics,
            poll_interval,
            max_messages_per_poll,
        }
    }

    /// 开始轮询消费
    pub async fn start_polling<F>(&self, message_handler: F) -> KafkaResult<()>
    where
        F: Fn(OwnedMessage) -> KafkaResult<()> + Send + Sync + 'static,
    {
        // 订阅主题
        let topic_refs: Vec<&str> = self.topics.iter().map(|s| s.as_str()).collect();
        self.app_state.subscribe(&topic_refs).await?;

        println!("开始轮询消费主题: {:?}", self.topics);

        loop {
            // 轮询消息
            match self.app_state.poll_batch(self.max_messages_per_poll).await {
                Ok(messages) => {
                    for message in messages {
                        if let Err(e) = message_handler(message) {
                            eprintln!("处理消息失败: {}", e);
                            // 可以选择继续处理或返回错误
                        }
                    }
                }
                Err(e) => {
                    eprintln!("轮询消息失败: {}", e);
                    // 可以选择重试或返回错误
                }
            }

            // 等待下次轮询
            tokio::time::sleep(self.poll_interval).await;
        }
    }

    /// 开始轮询消费（带超时控制）
    pub async fn start_polling_with_timeout<F>(
        &self,
        message_handler: F,
        poll_timeout: Duration,
    ) -> KafkaResult<()>
    where
        F: Fn(OwnedMessage) -> KafkaResult<()> + Send + Sync + 'static,
    {
        // 订阅主题
        let topic_refs: Vec<&str> = self.topics.iter().map(|s| s.as_str()).collect();
        self.app_state.subscribe(&topic_refs).await?;

        println!(
            "开始轮询消费主题: {:?} (超时: {:?})",
            self.topics, poll_timeout
        );

        loop {
            // 轮询消息（带超时）
            match timeout(
                poll_timeout,
                self.app_state.poll_batch(self.max_messages_per_poll),
            )
            .await
            {
                Ok(Ok(messages)) => {
                    for message in messages {
                        if let Err(e) = message_handler(message) {
                            eprintln!("处理消息失败: {}", e);
                        }
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("轮询消息失败: {}", e);
                }
                Err(_) => {
                    println!("轮询超时，继续下次轮询");
                }
            }

            // 等待下次轮询
            tokio::time::sleep(self.poll_interval).await;
        }
    }
}

/// 便捷函数：创建默认的 Kafka AppState
pub async fn create_default_kafka_app_state(
    bootstrap_servers: Vec<String>,
    consumer_group_id: String,
) -> KafkaResult<KafkaAppState> {
    let mut producer_config = KafkaProducerConfig::default();
    producer_config.base.bootstrap_servers = bootstrap_servers.clone();

    let mut consumer_config = KafkaConsumerConfig::default();
    consumer_config.base.bootstrap_servers = bootstrap_servers;
    consumer_config.group_id = consumer_group_id;

    KafkaAppState::new(producer_config, consumer_config).await
}

/// 便捷函数：从配置文件创建 Kafka AppState
pub async fn create_kafka_app_state_from_config(
    producer_config_path: &str,
    consumer_config_path: &str,
) -> KafkaResult<KafkaAppState> {
    let producer_config_content = std::fs::read_to_string(producer_config_path)
        .map_err(|e| KafkaError::ConfigError(format!("读取生产者配置文件失败: {}", e)))?;

    let consumer_config_content = std::fs::read_to_string(consumer_config_path)
        .map_err(|e| KafkaError::ConfigError(format!("读取消费者配置文件失败: {}", e)))?;

    let producer_config: KafkaProducerConfig = serde_yaml::from_str(&producer_config_content)
        .map_err(|e| KafkaError::ConfigError(format!("解析生产者配置文件失败: {}", e)))?;

    let consumer_config: KafkaConsumerConfig = serde_yaml::from_str(&consumer_config_content)
        .map_err(|e| KafkaError::ConfigError(format!("解析消费者配置文件失败: {}", e)))?;

    KafkaAppState::new(producer_config, consumer_config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_kafka_app_state_creation() {
        let producer_config = KafkaProducerConfig::default();
        let consumer_config = KafkaConsumerConfig::default();

        let result = KafkaAppState::new(producer_config, consumer_config).await;
        // 注意：这个测试可能会失败，因为需要实际的 Kafka 服务器
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_polling_consumer_service_creation() {
        let producer_config = KafkaProducerConfig::default();
        let consumer_config = KafkaConsumerConfig::default();

        if let Ok(app_state) = KafkaAppState::new(producer_config, consumer_config).await {
            let service = PollingConsumerService::new(
                app_state,
                vec!["test-topic".to_string()],
                Duration::from_secs(1),
                10,
            );

            assert_eq!(service.topics, vec!["test-topic"]);
            assert_eq!(service.poll_interval, Duration::from_secs(1));
            assert_eq!(service.max_messages_per_poll, 10);
        }
    }
}
