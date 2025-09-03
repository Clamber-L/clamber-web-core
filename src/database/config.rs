//! 数据库配置模块
//!
//! 定义数据库连接相关的配置结构，支持通过 clamber-core 的配置系统加载

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 数据库配置结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    /// 数据库连接 URL
    pub url: String,

    /// 最大连接数
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// 最小连接数
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// 连接超时时间（秒）
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_secs: u64,

    /// 获取连接超时时间（秒）
    #[serde(default = "default_acquire_timeout")]
    pub acquire_timeout_secs: u64,

    /// 空闲超时时间（秒）
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_secs: u64,

    /// 连接最大生命周期（秒）
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime_secs: u64,

    /// 是否启用 SQL 日志
    #[serde(default = "default_sql_logging")]
    pub sql_logging: bool,

    /// 慢查询阈值（毫秒）
    #[serde(default = "default_slow_threshold")]
    pub slow_threshold_ms: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "mysql://root:password@localhost:3306/clamber".to_string(),
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            connect_timeout_secs: default_connect_timeout(),
            acquire_timeout_secs: default_acquire_timeout(),
            idle_timeout_secs: default_idle_timeout(),
            max_lifetime_secs: default_max_lifetime(),
            sql_logging: default_sql_logging(),
            slow_threshold_ms: default_slow_threshold(),
        }
    }
}

impl DatabaseConfig {
    /// 获取连接超时时间
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_secs(self.connect_timeout_secs)
    }

    /// 获取获取连接超时时间
    pub fn acquire_timeout(&self) -> Duration {
        Duration::from_secs(self.acquire_timeout_secs)
    }

    /// 获取空闲超时时间
    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.idle_timeout_secs)
    }

    /// 获取连接最大生命周期
    pub fn max_lifetime(&self) -> Duration {
        Duration::from_secs(self.max_lifetime_secs)
    }

    /// 获取慢查询阈值
    pub fn slow_threshold(&self) -> Duration {
        Duration::from_millis(self.slow_threshold_ms)
    }

    /// 验证配置的有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.url.is_empty() {
            return Err("数据库 URL 不能为空".to_string());
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

        Ok(())
    }
}

// 默认值函数
fn default_max_connections() -> u32 {
    100
}
fn default_min_connections() -> u32 {
    5
}
fn default_connect_timeout() -> u64 {
    30
}
fn default_acquire_timeout() -> u64 {
    30
}
fn default_idle_timeout() -> u64 {
    600
}
fn default_max_lifetime() -> u64 {
    1800
}
fn default_sql_logging() -> bool {
    true
}
fn default_slow_threshold() -> u64 {
    1000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.min_connections, 5);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        let mut config = DatabaseConfig::default();

        // 测试空 URL
        config.url = String::new();
        assert!(config.validate().is_err());

        // 测试无效连接数
        config.url = "mysql://localhost/test".to_string();
        config.min_connections = 10;
        config.max_connections = 5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_duration_conversion() {
        let config = DatabaseConfig::default();
        assert_eq!(config.connect_timeout(), Duration::from_secs(30));
        assert_eq!(config.slow_threshold(), Duration::from_millis(1000));
    }
}
