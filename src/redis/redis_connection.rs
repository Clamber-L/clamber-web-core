//! Redis 连接模块
//!
//! 提供 Redis 连接的封装和扩展功能，支持连接池和基本操作

use crate::redis::{RedisConfig, RedisError, RedisResult};
use redis::{AsyncCommands, Client, ToRedisArgs, aio::ConnectionManager};
use std::time::Instant;
use tracing::{error, info, warn};

/// Redis 连接封装
#[derive(Clone)]
pub struct RedisConnection {
    /// Redis 连接管理器
    manager: ConnectionManager,
    /// 配置信息
    config: RedisConfig,
}

impl RedisConnection {
    /// 创建新的 Redis 连接
    pub async fn new(config: RedisConfig) -> RedisResult<Self> {
        // 验证配置
        config.validate().map_err(|msg| RedisError::config(msg))?;

        info!("正在连接 Redis: {}", mask_redis_url(&config.url));

        // 创建 Redis 客户端
        let client = Client::open(config.build_url()).map_err(|e| {
            error!("Redis 客户端创建失败: {}", e);
            RedisError::connection(format!("客户端创建失败: {}", e))
        })?;

        // 创建连接管理器
        let manager = ConnectionManager::new(client).await.map_err(|e| {
            error!("Redis 连接管理器创建失败: {}", e);
            RedisError::connection(format!("连接管理器创建失败: {}", e))
        })?;

        info!("Redis 连接成功建立");

        Ok(Self { manager, config })
    }

    /// 从 Redis URL 字符串创建连接（最常用）
    pub async fn from_url(redis_url: &str) -> RedisResult<Self> {
        info!("从 URL 创建 Redis 连接: {}", mask_redis_url(redis_url));
        let config = RedisConfig::from_url(redis_url);
        Self::new(config).await
    }

    /// 测试连接是否有效
    pub async fn ping(&mut self) -> RedisResult<()> {
        let start = Instant::now();

        redis::cmd("PING")
            .query_async::<String>(&mut self.manager)
            .await
            .map_err(|e| {
                warn!("Redis 连接测试失败: {}", e);
                RedisError::connection(format!("连接测试失败: {}", e))
            })?;

        let elapsed = start.elapsed();
        info!("Redis 连接测试成功，耗时: {:?}", elapsed);
        Ok(())
    }

    // =============================================================================
    // 使用 AsyncCommands trait 内置方法的示例（推荐）
    // =============================================================================

    /// 设置键值对 - 使用内置方法
    pub async fn set_builtin<K, V>(&mut self, key: K, value: V) -> RedisResult<()>
    where
        K: ToRedisArgs + Send + Sync,
        V: ToRedisArgs + Send + Sync,
    {
        let start = Instant::now();

        // 使用 AsyncCommands trait 的内置 set 方法
        let result = self.manager.set(key, value).await.map_err(RedisError::from);

        self.log_command_if_slow("SET_BUILTIN", start);
        result
    }

    /// 获取键的值 - 使用内置方法
    pub async fn get_builtin<K>(&mut self, key: K) -> RedisResult<Option<String>>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let start = Instant::now();

        // 使用 AsyncCommands trait 的内置 get 方法
        let result = self.manager.get(key).await.map_err(RedisError::from);

        self.log_command_if_slow("GET_BUILTIN", start);
        result
    }

    /// 检查键是否存在 - 使用内置方法
    pub async fn exists_builtin<K>(&mut self, key: K) -> RedisResult<bool>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let start = Instant::now();

        // 使用 AsyncCommands trait 的内置 exists 方法
        let result = self.manager.exists(key).await.map_err(RedisError::from);

        self.log_command_if_slow("EXISTS_BUILTIN", start);
        result
    }

    /// 列表操作：左侧推入
    pub async fn lpush<K, V>(&mut self, key: K, value: V) -> RedisResult<i32>
    where
        K: ToRedisArgs + Send + Sync,
        V: ToRedisArgs + Send + Sync,
    {
        let start = Instant::now();
        let result = self
            .manager
            .lpush(key, value)
            .await
            .map_err(RedisError::from);
        self.log_command_if_slow("LPUSH", start);
        result
    }

    /// 列表操作：右侧弹出
    pub async fn rpop<K>(&mut self, key: K) -> RedisResult<Option<String>>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let start = Instant::now();
        let result = self.manager.rpop(key, None).await.map_err(RedisError::from);
        self.log_command_if_slow("RPOP", start);
        result
    }

    /// 哈希操作：设置字段
    pub async fn hset<K, F, V>(&mut self, key: K, field: F, value: V) -> RedisResult<bool>
    where
        K: ToRedisArgs + Send + Sync,
        F: ToRedisArgs + Send + Sync,
        V: ToRedisArgs + Send + Sync,
    {
        let start = Instant::now();
        let result = self
            .manager
            .hset(key, field, value)
            .await
            .map_err(RedisError::from);
        self.log_command_if_slow("HSET", start);
        result
    }

    /// 哈希操作：获取字段
    pub async fn hget<K, F>(&mut self, key: K, field: F) -> RedisResult<Option<String>>
    where
        K: ToRedisArgs + Send + Sync,
        F: ToRedisArgs + Send + Sync,
    {
        let start = Instant::now();
        let result = self
            .manager
            .hget(key, field)
            .await
            .map_err(RedisError::from);
        self.log_command_if_slow("HGET", start);
        result
    }

    /// 获取连接统计信息
    pub fn get_stats(&self) -> RedisConnectionStats {
        RedisConnectionStats {
            max_connections: self.config.max_connections,
            min_connections: self.config.min_connections,
            connect_timeout: self.config.connect_timeout_secs,
            read_timeout: self.config.read_timeout_secs,
            write_timeout: self.config.write_timeout_secs,
        }
    }

    /// 记录慢命令日志
    fn log_command_if_slow(&self, command: &str, start: Instant) {
        if !self.config.command_logging {
            return;
        }

        let elapsed = start.elapsed();
        if elapsed > self.config.slow_threshold() {
            warn!("慢 Redis 命令: {} 耗时 {:?}", command, elapsed);
        }
    }
}

/// 便利函数：从 URL 创建连接（最常用）
pub async fn create_redis_connection_from_url(redis_url: &str) -> RedisResult<RedisConnection> {
    RedisConnection::from_url(redis_url).await
}

/// 便利函数：从配置对象创建连接
pub async fn create_redis_connection_from_config(
    config: RedisConfig,
) -> RedisResult<RedisConnection> {
    RedisConnection::new(config).await
}

/// 连接统计信息
#[derive(Debug, Clone)]
pub struct RedisConnectionStats {
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub read_timeout: u64,
    pub write_timeout: u64,
}

/// Redis 健康状态
#[derive(Debug, Clone)]
pub struct RedisHealthStatus {
    pub is_healthy: bool,
    pub response_time_ms: u64,
    pub message: String,
}

/// 屏蔽 Redis URL 中的敏感信息
pub fn mask_redis_url(url: &str) -> String {
    // 简单地屏蔽可能的密码部分
    if let Some(at_pos) = url.find('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            if let Some(slash_pos) = url[..colon_pos].rfind('/') {
                let before = &url[..slash_pos + 1];
                let after = &url[at_pos..];
                return format!("{}***:***{}", before, after);
            }
        }
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_redis_url() {
        let url = "redis://user:password@localhost:6379/0";
        let masked = mask_redis_url(url);
        assert!(masked.contains("***"));
        assert!(!masked.contains("password"));
    }

    #[test]
    fn test_connection_stats() {
        let config = RedisConfig::default();
        let stats = RedisConnectionStats {
            max_connections: config.max_connections,
            min_connections: config.min_connections,
            connect_timeout: config.connect_timeout_secs,
            read_timeout: config.read_timeout_secs,
            write_timeout: config.write_timeout_secs,
        };

        assert_eq!(stats.max_connections, 50);
        assert_eq!(stats.min_connections, 5);
    }
}
