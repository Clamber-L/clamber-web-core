//! Redis 配置模块
//!
//! 定义 Redis 连接相关的配置结构，支持通过 clamber-core 的配置系统加载

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Redis 配置结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisConfig {
    /// Redis 连接 URL
    pub url: String,

    /// 连接池最大连接数
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// 连接池最小连接数
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// 连接超时时间（秒）
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_secs: u64,

    /// 读取超时时间（秒）
    #[serde(default = "default_read_timeout")]
    pub read_timeout_secs: u64,

    /// 写入超时时间（秒）
    #[serde(default = "default_write_timeout")]
    pub write_timeout_secs: u64,

    /// 连接重试次数
    #[serde(default = "default_retry_attempts")]
    pub retry_attempts: u32,

    /// 重试间隔时间（毫秒）
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u64,

    /// 键的默认过期时间（秒，0表示不过期）
    #[serde(default = "default_default_ttl")]
    pub default_ttl_secs: u64,

    /// 是否启用 Redis 命令日志
    #[serde(default = "default_command_logging")]
    pub command_logging: bool,

    /// 慢命令阈值（毫秒）
    #[serde(default = "default_slow_threshold")]
    pub slow_threshold_ms: u64,

    /// 是否启用连接池
    #[serde(default = "default_enable_pool")]
    pub enable_pool: bool,

    /// 数据库索引
    #[serde(default = "default_database_index")]
    pub database_index: u8,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            connect_timeout_secs: default_connect_timeout(),
            read_timeout_secs: default_read_timeout(),
            write_timeout_secs: default_write_timeout(),
            retry_attempts: default_retry_attempts(),
            retry_delay_ms: default_retry_delay(),
            default_ttl_secs: default_default_ttl(),
            command_logging: default_command_logging(),
            slow_threshold_ms: default_slow_threshold(),
            enable_pool: default_enable_pool(),
            database_index: default_database_index(),
        }
    }
}

impl RedisConfig {
    /// 获取连接超时时间
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_secs(self.connect_timeout_secs)
    }

    /// 获取读取超时时间
    pub fn read_timeout(&self) -> Duration {
        Duration::from_secs(self.read_timeout_secs)
    }

    /// 获取写入超时时间
    pub fn write_timeout(&self) -> Duration {
        Duration::from_secs(self.write_timeout_secs)
    }

    /// 获取重试延迟时间
    pub fn retry_delay(&self) -> Duration {
        Duration::from_millis(self.retry_delay_ms)
    }

    /// 获取默认 TTL
    pub fn default_ttl(&self) -> Option<Duration> {
        if self.default_ttl_secs == 0 {
            None
        } else {
            Some(Duration::from_secs(self.default_ttl_secs))
        }
    }

    /// 获取慢命令阈值
    pub fn slow_threshold(&self) -> Duration {
        Duration::from_millis(self.slow_threshold_ms)
    }

    /// 验证配置的有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.url.is_empty() {
            return Err("Redis URL 不能为空".to_string());
        }

        if self.max_connections == 0 {
            return Err("最大连接数必须大于 0".to_string());
        }

        if self.min_connections > self.max_connections {
            return Err("最小连接数不能大于最大连接数".to_string());
        }

        if self.connect_timeout_secs == 0 {
            return Err("连接超时时间必须大于 0".to_string());
        }

        if self.read_timeout_secs == 0 {
            return Err("读取超时时间必须大于 0".to_string());
        }

        if self.write_timeout_secs == 0 {
            return Err("写入超时时间必须大于 0".to_string());
        }

        if self.retry_delay_ms == 0 {
            return Err("重试延迟时间必须大于 0".to_string());
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

// 默认值函数
fn default_max_connections() -> u32 {
    50
}

fn default_min_connections() -> u32 {
    5
}

fn default_connect_timeout() -> u64 {
    10
}

fn default_read_timeout() -> u64 {
    30
}

fn default_write_timeout() -> u64 {
    30
}

fn default_retry_attempts() -> u32 {
    3
}

fn default_retry_delay() -> u64 {
    100
}

fn default_default_ttl() -> u64 {
    0 // 不过期
}

fn default_command_logging() -> bool {
    true
}

fn default_slow_threshold() -> u64 {
    100
}

fn default_enable_pool() -> bool {
    true
}

fn default_database_index() -> u8 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RedisConfig::default();
        assert_eq!(config.max_connections, 50);
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.database_index, 0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        let mut config = RedisConfig::default();

        // 测试空 URL
        config.url = String::new();
        assert!(config.validate().is_err());

        // 测试无效连接数
        config.url = "redis://localhost:6379".to_string();
        config.min_connections = 10;
        config.max_connections = 5;
        assert!(config.validate().is_err());

        // 测试无效超时时间
        config.min_connections = 5;
        config.max_connections = 10;
        config.connect_timeout_secs = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_duration_conversion() {
        let config = RedisConfig::default();
        assert_eq!(config.connect_timeout(), Duration::from_secs(10));
        assert_eq!(config.slow_threshold(), Duration::from_millis(100));
        assert_eq!(config.default_ttl(), None);
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

    #[test]
    fn test_from_url() {
        let config = RedisConfig::from_url("redis://user:pass@example.com:6379");
        assert_eq!(config.url, "redis://user:pass@example.com:6379");
        assert_eq!(config.max_connections, 50);
    }

    #[test]
    fn test_ttl_handling() {
        let mut config = RedisConfig::default();

        // 测试无 TTL
        config.default_ttl_secs = 0;
        assert_eq!(config.default_ttl(), None);

        // 测试有 TTL
        config.default_ttl_secs = 3600;
        assert_eq!(config.default_ttl(), Some(Duration::from_secs(3600)));
    }
}
