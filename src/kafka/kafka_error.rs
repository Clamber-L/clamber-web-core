//! Kafka 错误处理模块
//!
//! 提供统一的 Kafka 错误类型和处理机制

use thiserror::Error;

/// Kafka 相关错误类型
#[derive(Error, Debug)]
pub enum KafkaError {
    /// 配置错误
    #[error("Kafka配置错误: {0}")]
    ConfigError(String),

    /// 生产者错误
    #[error("Kafka生产者错误: {0}")]
    ProducerError(String),

    /// 消费者错误
    #[error("Kafka消费者错误: {0}")]
    ConsumerError(String),

    /// 消息发送错误
    #[error("消息发送失败: {0}")]
    SendError(String),

    /// 消息接收错误
    #[error("消息接收失败: {0}")]
    ReceiveError(String),

    /// 序列化错误
    #[error("消息序列化失败: {0}")]
    SerializationError(String),

    /// 反序列化错误
    #[error("消息反序列化失败: {0}")]
    DeserializationError(String),

    /// 连接错误
    #[error("Kafka连接失败: {0}")]
    ConnectionError(String),

    /// 超时错误
    #[error("操作超时: {0}")]
    TimeoutError(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    InternalError(String),
}

impl From<rdkafka::error::KafkaError> for KafkaError {
    fn from(err: rdkafka::error::KafkaError) -> Self {
        match err {
            rdkafka::error::KafkaError::MessageProduction(code) => {
                KafkaError::ProducerError(format!("消息生产错误: {:?}", code))
            }
            rdkafka::error::KafkaError::MessageConsumption(code) => {
                KafkaError::ConsumerError(format!("消息消费错误: {:?}", code))
            }
            rdkafka::error::KafkaError::ClientCreation(msg) => KafkaError::ConnectionError(msg),
            _ => KafkaError::InternalError(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for KafkaError {
    fn from(err: serde_json::Error) -> Self {
        KafkaError::SerializationError(err.to_string())
    }
}

impl From<std::time::SystemTimeError> for KafkaError {
    fn from(err: std::time::SystemTimeError) -> Self {
        KafkaError::InternalError(err.to_string())
    }
}

/// Kafka 结果类型
pub type KafkaResult<T> = Result<T, KafkaError>;
