//! Clamber Web Core
//!
//! 基于 clamber-core 构建的 Web 基础 crate，提供：
//! - 数据库管理（基于 SeaORM） - 启用 `database` feature
//! - Redis 连接池管理 - 启用 `redis` feature  
//! - Kafka 消息队列支持 - 启用 `kafka` feature
//! - Web 框架集成（基于 Axum）
//! - 认证和授权
//! - 统一错误处理
//! - 配置管理
//!
//! ## Features
//!
//! - `database`: 启用数据库模块（SeaORM）
//! - `redis`: 启用Redis模块
//! - `kafka`: 启用Kafka模块
//! - `full`: 启用所有功能
//! - `default`: 默认启用所有功能
//!
//! ## 使用示例
//!
//! ```toml
//! [dependencies]
//! clamber-web-core = { version = "0.1.1", features = ["database", "redis"] }
//! ```

#[cfg(feature = "database")]
pub mod database;

#[cfg(feature = "redis")]
pub mod redis;

#[cfg(feature = "kafka")]
pub mod kafka;

// 重新导出主要模块
#[cfg(feature = "database")]
pub use database::*;

#[cfg(feature = "redis")]
pub use redis::*;

#[cfg(feature = "kafka")]
pub use kafka::*;

// 重新导出核心依赖
pub use axum;
pub use chrono;
pub use serde;
pub use serde_json;
pub use tokio;

// 条件性重新导出可选依赖
#[cfg(feature = "database")]
pub use clamber_core;
#[cfg(feature = "database")]
pub use sea_orm;

#[cfg(feature = "redis")]
pub use redis as redis_crate;

#[cfg(feature = "kafka")]
pub use rdkafka;
