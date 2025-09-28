//! Redis 配置模块
//!
//! 定义 Redis 连接相关的配置结构，支持通过 clamber-core 的配置系统加载

use serde::{Deserialize, Serialize};

/// Redis 配置结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisConfig {
    /// Redis 连接 URL
    pub url: String,

    /// 数据库索引
    #[serde(default = "default_database_index")]
    pub database_index: u8,

    /// 连接超时时间（秒）
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_secs: u64,

    /// 响应超时时间（秒）
    #[serde(default = "default_response_timeout")]
    pub response_timeout_secs: u64,

    /// 重试次数
    #[serde(default = "default_retry_count")]
    pub retry_count: usize,

    /// 重试延迟因子（毫秒）
    #[serde(default = "default_retry_factor")]
    pub retry_factor_ms: u64,

    /// 最大重试延迟（毫秒）
    #[serde(default = "default_max_retry_delay")]
    pub max_retry_delay_ms: u64,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            database_index: default_database_index(),
            connection_timeout_secs: default_connection_timeout(),
            response_timeout_secs: default_response_timeout(),
            retry_count: default_retry_count(),
            retry_factor_ms: default_retry_factor(),
            max_retry_delay_ms: default_max_retry_delay(),
        }
    }
}

impl RedisConfig {
    /// 验证配置的有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.url.is_empty() {
            return Err("Redis URL 不能为空".to_string());
        }
        Ok(())
    }

    /// 构建 Redis URL，包含数据库索引
    pub fn build_url(&self) -> String {
        if self.database_index == 0 {
            self.url.clone()
        } else {
            format!("{}/{}", self.url.trim_end_matches('/'), self.database_index)
        }
    }

    /// 从 URL 创建简单配置
    pub fn from_url(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }
}

fn default_database_index() -> u8 {
    0
}

fn default_connection_timeout() -> u64 {
    30
}

fn default_response_timeout() -> u64 {
    0 // 0 表示使用默认值（无超时）
}

fn default_retry_count() -> usize {
    6
}

fn default_retry_factor() -> u64 {
    100
}

fn default_max_retry_delay() -> u64 {
    0 // 0 表示使用默认值（无最大延迟限制）
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RedisConfig::default();
        assert_eq!(config.database_index, 0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        let mut config = RedisConfig::default();

        // 测试空 URL
        config.url = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_url_building() {
        let mut config = RedisConfig::default();
        config.url = "redis://localhost:6379".to_string();

        // 测试默认数据库索引
        assert_eq!(config.build_url(), "redis://localhost:6379");

        // 测试指定数据库索引
        config.database_index = 1;
        assert_eq!(config.build_url(), "redis://localhost:6379/1");
    }
}
