//! 数据库模块
//!
//! 提供基于 SeaORM 的数据库连接管理、配置和工具函数
//! 集成 clamber-core 的配置管理功能

pub mod config;
pub mod connection;
pub mod error;

// 重新导出主要组件
pub use config::DatabaseConfig;
pub use connection::{ConnectionStats, SeaOrmConnection, HealthStatus};
pub use error::{DatabaseError, DatabaseResult};

// 便利函数
pub use connection::{
    // 用于 Axum AppState 的 Arc 包装版本
    create_connection_from_config,
    create_connection_from_url,
};
