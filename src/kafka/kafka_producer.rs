//! Kafka 生产者服务模块
//!
//! 提供 Kafka 消息发送功能

use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use rdkafka::util::Timeout;
use serde::Serialize;
use std::time::Duration;

use crate::kafka::kafka_config::KafkaProducerConfig;
use crate::kafka::kafka_error::{KafkaError, KafkaResult};

/// Kafka 生产者服务
pub struct KafkaProducer {
    producer: FutureProducer,
    config: KafkaProducerConfig,
}

impl KafkaProducer {
    /// 创建新的 Kafka 生产者
    pub fn new(config: KafkaProducerConfig) -> KafkaResult<Self> {
        let producer_config = config.to_producer_config()?;
        let producer: FutureProducer = producer_config
            .create()
            .map_err(|e| KafkaError::ProducerError(format!("创建生产者失败: {}", e)))?;

        Ok(Self { producer, config })
    }

    /// 发送文本消息
    pub async fn send_message(
        &self,
        topic: &str,
        key: Option<&str>,
        payload: &str,
    ) -> KafkaResult<()> {
        self.send_bytes(topic, key, payload.as_bytes()).await
    }

    /// 发送字节消息
    pub async fn send_bytes(
        &self,
        topic: &str,
        key: Option<&str>,
        payload: &[u8],
    ) -> KafkaResult<()> {
        let mut record = FutureRecord::to(topic).payload(payload);

        if let Some(key) = key {
            record = record.key(key);
        }

        let timeout = Duration::from_millis(self.config.base.request_timeout_ms.unwrap_or(30000));

        let result = self.producer.send(record, Timeout::After(timeout)).await;

        match result {
            Ok(_) => Ok(()),
            Err((kafka_error, _)) => Err(KafkaError::from(kafka_error)),
        }
    }

    /// 发送序列化的消息
    pub async fn send_serialized<T: Serialize>(
        &self,
        topic: &str,
        key: Option<&str>,
        data: &T,
    ) -> KafkaResult<()> {
        let payload =
            serde_json::to_vec(data).map_err(|e| KafkaError::SerializationError(e.to_string()))?;

        self.send_bytes(topic, key, &payload).await
    }

    /// 发送带分区的消息
    pub async fn send_to_partition(
        &self,
        topic: &str,
        partition: i32,
        key: Option<&str>,
        payload: &[u8],
    ) -> KafkaResult<()> {
        let mut record = FutureRecord::to(topic)
            .partition(partition)
            .payload(payload);

        if let Some(key) = key {
            record = record.key(key);
        }

        let timeout = Duration::from_millis(self.config.base.request_timeout_ms.unwrap_or(30000));

        let result = self.producer.send(record, Timeout::After(timeout)).await;

        match result {
            Ok(_) => Ok(()),
            Err((kafka_error, _)) => Err(KafkaError::from(kafka_error)),
        }
    }

    /// 批量发送消息
    pub async fn send_batch(
        &self,
        topic: &str,
        messages: Vec<(Option<String>, Vec<u8>)>,
    ) -> KafkaResult<()> {
        let timeout = Duration::from_millis(self.config.base.request_timeout_ms.unwrap_or(30000));

        for (key, payload) in messages {
            let mut record = FutureRecord::to(topic).payload(&payload);

            if let Some(ref key) = key {
                record = record.key(key);
            }

            let result = self.producer.send(record, Timeout::After(timeout)).await;

            match result {
                Ok(_) => {}
                Err((kafka_error, _)) => return Err(KafkaError::from(kafka_error)),
            }
        }

        Ok(())
    }

    /// 刷新生产者缓冲区
    pub async fn flush(&self) -> KafkaResult<()> {
        let timeout = Duration::from_millis(self.config.base.request_timeout_ms.unwrap_or(30000));

        self.producer
            .flush(timeout)
            .map_err(|e| KafkaError::ProducerError(format!("刷新缓冲区失败: {}", e)))?;

        Ok(())
    }

    /// 获取生产者配置
    pub fn get_config(&self) -> &KafkaProducerConfig {
        &self.config
    }

    /// 获取生产者统计信息
    pub fn get_stats(&self) -> KafkaResult<String> {
        // 注意：在新版本的 rdkafka 中，统计信息的获取方式可能有所不同
        // 这里返回一个占位符，实际使用时需要根据具体版本调整
        Ok("统计信息功能暂未实现".to_string())
    }
}

/// 事务性 Kafka 生产者
pub struct TransactionalKafkaProducer {
    producer: FutureProducer,
    config: KafkaProducerConfig,
    transaction_id: String,
}

impl TransactionalKafkaProducer {
    /// 创建新的事务性 Kafka 生产者
    pub fn new(config: KafkaProducerConfig, transaction_id: String) -> KafkaResult<Self> {
        let mut producer_config = config.to_producer_config()?;
        producer_config.set("transactional.id", &transaction_id);
        producer_config.set("enable.idempotence", "true");

        let producer: FutureProducer = producer_config
            .create()
            .map_err(|e| KafkaError::ProducerError(format!("创建事务性生产者失败: {}", e)))?;

        Ok(Self {
            producer,
            config,
            transaction_id,
        })
    }

    /// 初始化事务
    pub async fn init_transaction(&self) -> KafkaResult<()> {
        self.producer
            .init_transactions(Duration::from_millis(
                self.config.transaction_timeout_ms.unwrap_or(60000),
            ))
            .map_err(|e| KafkaError::ProducerError(format!("初始化事务失败: {}", e)))?;

        Ok(())
    }

    /// 开始事务
    pub async fn begin_transaction(&self) -> KafkaResult<()> {
        self.producer
            .begin_transaction()
            .map_err(|e| KafkaError::ProducerError(format!("开始事务失败: {}", e)))?;

        Ok(())
    }

    /// 提交事务
    pub async fn commit_transaction(&self) -> KafkaResult<()> {
        self.producer
            .commit_transaction(Duration::from_millis(
                self.config.transaction_timeout_ms.unwrap_or(60000),
            ))
            .map_err(|e| KafkaError::ProducerError(format!("提交事务失败: {}", e)))?;

        Ok(())
    }

    /// 中止事务
    pub async fn abort_transaction(&self) -> KafkaResult<()> {
        self.producer
            .abort_transaction(Duration::from_millis(
                self.config.transaction_timeout_ms.unwrap_or(60000),
            ))
            .map_err(|e| KafkaError::ProducerError(format!("中止事务失败: {}", e)))?;

        Ok(())
    }

    /// 发送事务性消息
    pub async fn send_transactional_message(
        &self,
        topic: &str,
        key: Option<&str>,
        payload: &[u8],
    ) -> KafkaResult<()> {
        let mut record = FutureRecord::to(topic).payload(payload);

        if let Some(key) = key {
            record = record.key(key);
        }

        let timeout = Duration::from_millis(self.config.base.request_timeout_ms.unwrap_or(30000));

        let result = self.producer.send(record, Timeout::After(timeout)).await;

        match result {
            Ok(_) => Ok(()),
            Err((kafka_error, _)) => Err(KafkaError::from(kafka_error)),
        }
    }

    /// 获取事务ID
    pub fn get_transaction_id(&self) -> &str {
        &self.transaction_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_producer_config_creation() {
        let config = KafkaProducerConfig::default();
        assert!(config.to_producer_config().is_ok());
    }

    #[test]
    fn test_transactional_producer_config() {
        let mut config = KafkaProducerConfig::default();
        config.transactional_id = Some("test-transaction".to_string());
        config.enable_idempotence = Some(true);

        let result = TransactionalKafkaProducer::new(config, "test-transaction".to_string());
        // 注意：这个测试可能会失败，因为需要实际的 Kafka 服务器
        // 在实际测试中，应该使用嵌入式 Kafka 或测试容器
        assert!(result.is_err() || result.is_ok());
    }
}
