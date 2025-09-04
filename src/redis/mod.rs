//! Redis 模块
//!
//! 提供基于 Redis 的缓存连接管理、配置和工具函数
//! 集成 clamber-core 的配置管理功能

pub mod redis_config;
pub mod redis_connection;
pub mod redis_error;

// 重新导出主要组件
pub use redis_config::RedisConfig;
pub use redis_connection::{RedisConnection, RedisConnectionStats, RedisHealthStatus};
pub use redis_error::{RedisError, RedisResult};

// 便利函数
pub use redis_connection::{
    // 用于 Axum AppState 的便利版本
    create_redis_connection_from_config,
    create_redis_connection_from_url,
};
