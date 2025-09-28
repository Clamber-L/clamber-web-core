//! Kafka 配置模块
//!
//! 提供 Kafka 生产者和消费者的配置管理

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::kafka::kafka_error::KafkaResult;

/// Kafka 基础配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaBaseConfig {
    /// Kafka 服务器地址列表
    pub bootstrap_servers: Vec<String>,
    /// 客户端ID
    pub client_id: Option<String>,
    /// 安全协议 (PLAINTEXT, SSL, SASL_PLAINTEXT, SASL_SSL)
    pub security_protocol: Option<String>,
    /// SASL 机制 (PLAIN, SCRAM-SHA-256, SCRAM-SHA-512)
    pub sasl_mechanism: Option<String>,
    /// SASL 用户名
    pub sasl_username: Option<String>,
    /// SASL 密码
    pub sasl_password: Option<String>,
    /// SSL CA 证书路径
    pub ssl_ca_location: Option<String>,
    /// SSL 客户端证书路径
    pub ssl_certificate_location: Option<String>,
    /// SSL 客户端私钥路径
    pub ssl_key_location: Option<String>,
    /// 连接超时时间（毫秒）
    pub connection_timeout_ms: Option<u64>,
    /// 请求超时时间（毫秒）
    pub request_timeout_ms: Option<u64>,
    /// 自定义配置参数
    pub custom_configs: Option<HashMap<String, String>>,
}

impl Default for KafkaBaseConfig {
    fn default() -> Self {
        Self {
            bootstrap_servers: vec!["localhost:9092".to_string()],
            client_id: Some("clamber-kafka-client".to_string()),
            security_protocol: Some("PLAINTEXT".to_string()),
            sasl_mechanism: None,
            sasl_username: None,
            sasl_password: None,
            ssl_ca_location: None,
            ssl_certificate_location: None,
            ssl_key_location: None,
            connection_timeout_ms: Some(30000),
            request_timeout_ms: Some(30000),
            custom_configs: None,
        }
    }
}

/// Kafka 生产者配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaProducerConfig {
    /// 基础配置
    pub base: KafkaBaseConfig,
    /// 确认模式 (0: 不等待, 1: 等待leader确认, -1: 等待所有副本确认)
    pub acks: Option<String>,
    /// 重试次数
    pub retries: Option<i32>,
    /// 重试间隔（毫秒）
    pub retry_backoff_ms: Option<u64>,
    /// 批量发送大小（字节）
    pub batch_size: Option<i32>,
    /// 批量发送延迟（毫秒）
    pub linger_ms: Option<u64>,
    /// 压缩类型 (none, gzip, snappy, lz4, zstd)
    pub compression_type: Option<String>,
    /// 最大请求大小（字节）
    pub max_request_size: Option<i32>,
    /// 发送缓冲区大小（字节）
    pub send_buffer_bytes: Option<i32>,
    /// 接收缓冲区大小（字节）
    pub receive_buffer_bytes: Option<i32>,
    /// 幂等性
    pub enable_idempotence: Option<bool>,
    /// 事务ID（用于事务性生产者）
    pub transactional_id: Option<String>,
    /// 事务超时时间（毫秒）
    pub transaction_timeout_ms: Option<u64>,
}

impl Default for KafkaProducerConfig {
    fn default() -> Self {
        Self {
            base: KafkaBaseConfig::default(),
            acks: Some("1".to_string()),
            retries: Some(3),
            retry_backoff_ms: Some(100),
            batch_size: Some(16384),
            linger_ms: Some(0),
            compression_type: Some("none".to_string()),
            max_request_size: None,     // 移除可能有问题的配置
            send_buffer_bytes: None,    // 移除可能有问题的配置
            receive_buffer_bytes: None, // 移除可能有问题的配置
            enable_idempotence: Some(false),
            transactional_id: None,
            transaction_timeout_ms: Some(60000),
        }
    }
}

/// Kafka 消费者配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaConsumerConfig {
    /// 基础配置
    pub base: KafkaBaseConfig,
    /// 消费者组ID
    pub group_id: String,
    /// 自动提交偏移量
    pub enable_auto_commit: Option<bool>,
    /// 自动提交间隔（毫秒）
    pub auto_commit_interval_ms: Option<u64>,
    /// 会话超时时间（毫秒）
    pub session_timeout_ms: Option<u64>,
    /// 心跳间隔（毫秒）
    pub heartbeat_interval_ms: Option<u64>,
    /// 最大轮询间隔（毫秒）
    pub max_poll_interval_ms: Option<u64>,
    /// 最大轮询记录数
    pub max_poll_records: Option<i32>,
    /// 分区分配策略 (range, roundrobin, sticky)
    pub partition_assignment_strategy: Option<String>,
    /// 偏移量重置策略 (earliest, latest, none)
    pub auto_offset_reset: Option<String>,
    /// 获取最小字节数
    pub fetch_min_bytes: Option<i32>,
    /// 获取最大字节数
    pub fetch_max_bytes: Option<i32>,
    /// 获取等待时间（毫秒）
    pub fetch_max_wait_ms: Option<u64>,
    /// 最大分区获取字节数
    pub max_partition_fetch_bytes: Option<i32>,
    /// 隔离级别 (read_uncommitted, read_committed)
    pub isolation_level: Option<String>,
}

impl Default for KafkaConsumerConfig {
    fn default() -> Self {
        Self {
            base: KafkaBaseConfig::default(),
            group_id: "clamber-consumer-group".to_string(),
            enable_auto_commit: Some(true),
            auto_commit_interval_ms: Some(5000),
            session_timeout_ms: Some(30000),
            heartbeat_interval_ms: Some(3000),
            max_poll_interval_ms: Some(300000),
            max_poll_records: None, // 移除可能有问题的配置
            partition_assignment_strategy: Some("range".to_string()),
            auto_offset_reset: Some("latest".to_string()),
            fetch_min_bytes: Some(1),
            fetch_max_bytes: None,           // 移除可能有问题的配置
            fetch_max_wait_ms: None,         // 移除可能有问题的配置
            max_partition_fetch_bytes: None, // 移除可能有问题的配置
            isolation_level: Some("read_uncommitted".to_string()),
        }
    }
}

impl KafkaBaseConfig {
    /// 转换为 rdkafka 客户端配置
    pub fn to_client_config(&self) -> KafkaResult<rdkafka::ClientConfig> {
        let mut config = rdkafka::ClientConfig::new();

        // 设置基础配置
        config.set("bootstrap.servers", self.bootstrap_servers.join(","));

        if let Some(client_id) = &self.client_id {
            config.set("client.id", client_id);
        }

        if let Some(protocol) = &self.security_protocol {
            config.set("security.protocol", protocol);
        }

        if let Some(mechanism) = &self.sasl_mechanism {
            config.set("sasl.mechanism", mechanism);
        }

        if let Some(username) = &self.sasl_username {
            config.set("sasl.username", username);
        }

        if let Some(password) = &self.sasl_password {
            config.set("sasl.password", password);
        }

        if let Some(ca_location) = &self.ssl_ca_location {
            config.set("ssl.ca.location", ca_location);
        }

        if let Some(cert_location) = &self.ssl_certificate_location {
            config.set("ssl.certificate.location", cert_location);
        }

        if let Some(key_location) = &self.ssl_key_location {
            config.set("ssl.key.location", key_location);
        }

        if let Some(timeout) = self.connection_timeout_ms {
            config.set("connections.max.idle.ms", timeout.to_string());
        }

        if let Some(timeout) = self.request_timeout_ms {
            config.set("request.timeout.ms", timeout.to_string());
        }

        // 设置自定义配置
        if let Some(custom_configs) = &self.custom_configs {
            for (key, value) in custom_configs {
                config.set(key, value);
            }
        }

        Ok(config)
    }
}

impl KafkaProducerConfig {
    /// 转换为 rdkafka 客户端配置（用于生产者）
    pub fn to_producer_config(&self) -> KafkaResult<rdkafka::ClientConfig> {
        let mut config = self.base.to_client_config()?;

        // 设置生产者特定配置
        if let Some(acks) = &self.acks {
            config.set("acks", acks);
        }

        if let Some(retries) = self.retries {
            config.set("retries", retries.to_string());
        }

        if let Some(backoff) = self.retry_backoff_ms {
            config.set("retry.backoff.ms", backoff.to_string());
        }

        if let Some(batch_size) = self.batch_size {
            config.set("batch.size", batch_size.to_string());
        }

        if let Some(linger) = self.linger_ms {
            config.set("linger.ms", linger.to_string());
        }

        if let Some(compression) = &self.compression_type {
            config.set("compression.type", compression);
        }

        if let Some(max_size) = self.max_request_size {
            config.set("message.max.bytes", max_size.to_string());
        }

        if let Some(send_buffer) = self.send_buffer_bytes {
            config.set("socket.send.buffer.bytes", send_buffer.to_string());
        }

        if let Some(receive_buffer) = self.receive_buffer_bytes {
            config.set("socket.receive.buffer.bytes", receive_buffer.to_string());
        }

        if let Some(idempotence) = self.enable_idempotence {
            config.set("enable.idempotence", idempotence.to_string());
        }

        if let Some(transactional_id) = &self.transactional_id {
            config.set("transactional.id", transactional_id);
        }

        if let Some(timeout) = self.transaction_timeout_ms {
            config.set("transaction.timeout.ms", timeout.to_string());
        }

        Ok(config)
    }
}

impl KafkaConsumerConfig {
    /// 转换为 rdkafka 客户端配置（用于消费者）
    pub fn to_consumer_config(&self) -> KafkaResult<rdkafka::ClientConfig> {
        let mut config = self.base.to_client_config()?;

        // 设置消费者特定配置
        config.set("group.id", &self.group_id);

        if let Some(auto_commit) = self.enable_auto_commit {
            config.set("enable.auto.commit", auto_commit.to_string());
        }

        if let Some(interval) = self.auto_commit_interval_ms {
            config.set("auto.commit.interval.ms", interval.to_string());
        }

        if let Some(timeout) = self.session_timeout_ms {
            config.set("session.timeout.ms", timeout.to_string());
        }

        if let Some(interval) = self.heartbeat_interval_ms {
            config.set("heartbeat.interval.ms", interval.to_string());
        }

        if let Some(interval) = self.max_poll_interval_ms {
            config.set("max.poll.interval.ms", interval.to_string());
        }

        if let Some(records) = self.max_poll_records {
            config.set("max.poll.records", records.to_string());
        }

        if let Some(strategy) = &self.partition_assignment_strategy {
            config.set("partition.assignment.strategy", strategy);
        }

        if let Some(reset) = &self.auto_offset_reset {
            config.set("auto.offset.reset", reset);
        }

        if let Some(min_bytes) = self.fetch_min_bytes {
            config.set("fetch.min.bytes", min_bytes.to_string());
        }

        if let Some(max_bytes) = self.fetch_max_bytes {
            config.set("fetch.max.bytes", max_bytes.to_string());
        }

        if let Some(wait_ms) = self.fetch_max_wait_ms {
            config.set("fetch.wait.max.ms", wait_ms.to_string());
        }

        if let Some(partition_bytes) = self.max_partition_fetch_bytes {
            config.set("max.partition.fetch.bytes", partition_bytes.to_string());
        }

        if let Some(isolation) = &self.isolation_level {
            config.set("isolation.level", isolation);
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_producer_config() {
        let config = KafkaProducerConfig::default();
        assert_eq!(config.base.bootstrap_servers, vec!["localhost:9092"]);
        assert_eq!(config.acks, Some("1".to_string()));
        assert_eq!(config.retries, Some(3));
    }

    #[test]
    fn test_default_consumer_config() {
        let config = KafkaConsumerConfig::default();
        assert_eq!(config.group_id, "clamber-consumer-group");
        assert_eq!(config.enable_auto_commit, Some(true));
    }

    #[test]
    fn test_config_serialization() {
        let producer_config = KafkaProducerConfig::default();
        let serialized = serde_json::to_string(&producer_config).unwrap();
        let deserialized: KafkaProducerConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(producer_config.acks, deserialized.acks);
    }
}
