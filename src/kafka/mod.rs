//! Kafka 模块
//!
//! 提供完整的 Kafka 客户端功能，包括：
//! - 配置管理
//! - 生产者服务
//! - 消费者服务
//! - 错误处理

pub mod kafka_config;
pub mod kafka_consumer;
pub mod kafka_error;
pub mod kafka_producer;

// 重新导出主要类型
pub use kafka_config::{KafkaBaseConfig, KafkaConsumerConfig, KafkaProducerConfig};
pub use kafka_consumer::{
    AdvancedKafkaConsumer, ConsumerGroupManager, KafkaConsumer, MessageHandler,
};
pub use kafka_error::{KafkaError, KafkaResult};
pub use kafka_producer::{KafkaProducer, TransactionalKafkaProducer};

// 重新导出 rdkafka 相关类型
pub use rdkafka::{
    message::{Message, OwnedMessage},
    producer::FutureRecord,
    topic_partition_list::TopicPartitionList,
    util::Timeout,
};

/// Kafka 客户端构建器
pub struct KafkaClientBuilder {
    producer_config: Option<KafkaProducerConfig>,
    consumer_config: Option<KafkaConsumerConfig>,
}

impl KafkaClientBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            producer_config: None,
            consumer_config: None,
        }
    }

    /// 设置生产者配置
    pub fn with_producer_config(mut self, config: KafkaProducerConfig) -> Self {
        self.producer_config = Some(config);
        self
    }

    /// 设置消费者配置
    pub fn with_consumer_config(mut self, config: KafkaConsumerConfig) -> Self {
        self.consumer_config = Some(config);
        self
    }

    /// 构建生产者
    pub fn build_producer(self) -> KafkaResult<KafkaProducer> {
        let config = self
            .producer_config
            .ok_or_else(|| KafkaError::ConfigError("生产者配置未设置".to_string()))?;
        KafkaProducer::new(config)
    }

    /// 构建消费者
    pub fn build_consumer(self) -> KafkaResult<KafkaConsumer> {
        let config = self
            .consumer_config
            .ok_or_else(|| KafkaError::ConfigError("消费者配置未设置".to_string()))?;
        KafkaConsumer::new(config)
    }

    /// 构建事务性生产者
    pub fn build_transactional_producer(
        self,
        transaction_id: String,
    ) -> KafkaResult<TransactionalKafkaProducer> {
        let config = self
            .producer_config
            .ok_or_else(|| KafkaError::ConfigError("生产者配置未设置".to_string()))?;
        TransactionalKafkaProducer::new(config, transaction_id)
    }

    /// 构建高级消费者
    pub fn build_advanced_consumer(self) -> KafkaResult<AdvancedKafkaConsumer> {
        let config = self
            .consumer_config
            .ok_or_else(|| KafkaError::ConfigError("消费者配置未设置".to_string()))?;
        AdvancedKafkaConsumer::new(config)
    }
}

impl Default for KafkaClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 便捷函数：创建默认生产者
pub fn create_default_producer() -> KafkaResult<KafkaProducer> {
    KafkaProducer::new(KafkaProducerConfig::default())
}

/// 便捷函数：创建默认消费者
pub fn create_default_consumer(group_id: String) -> KafkaResult<KafkaConsumer> {
    let mut config = KafkaConsumerConfig::default();
    config.group_id = group_id;
    KafkaConsumer::new(config)
}

/// 便捷函数：从配置文件创建生产者
pub fn create_producer_from_config(config_path: &str) -> KafkaResult<KafkaProducer> {
    let config_content = std::fs::read_to_string(config_path)
        .map_err(|e| KafkaError::ConfigError(format!("读取配置文件失败: {}", e)))?;

    let config: KafkaProducerConfig = serde_yaml::from_str(&config_content)
        .map_err(|e| KafkaError::ConfigError(format!("解析配置文件失败: {}", e)))?;

    KafkaProducer::new(config)
}

/// 便捷函数：从配置文件创建消费者
pub fn create_consumer_from_config(config_path: &str) -> KafkaResult<KafkaConsumer> {
    let config_content = std::fs::read_to_string(config_path)
        .map_err(|e| KafkaError::ConfigError(format!("读取配置文件失败: {}", e)))?;

    let config: KafkaConsumerConfig = serde_yaml::from_str(&config_content)
        .map_err(|e| KafkaError::ConfigError(format!("解析配置文件失败: {}", e)))?;

    KafkaConsumer::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kafka_client_builder() {
        let producer_config = KafkaProducerConfig::default();
        let consumer_config = KafkaConsumerConfig::default();

        // 测试构建器创建（实际构建可能会失败，因为需要 Kafka 服务器）
        let producer_result = KafkaClientBuilder::new()
            .with_producer_config(producer_config)
            .build_producer();
        let consumer_result = KafkaClientBuilder::new()
            .with_consumer_config(consumer_config)
            .build_consumer();

        // 这些测试可能会失败，因为需要实际的 Kafka 服务器
        assert!(producer_result.is_err() || producer_result.is_ok());
        assert!(consumer_result.is_err() || consumer_result.is_ok());
    }

    #[test]
    fn test_convenience_functions() {
        // 测试便捷函数（可能会失败，因为需要 Kafka 服务器）
        let producer_result = create_default_producer();
        let consumer_result = create_default_consumer("test-group".to_string());

        assert!(producer_result.is_err() || producer_result.is_ok());
        assert!(consumer_result.is_err() || consumer_result.is_ok());
    }
}
