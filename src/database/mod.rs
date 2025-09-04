//! 数据库模块
//!
//! 提供基于 SeaORM 的数据库连接管理、配置和工具函数
//! 集成 clamber-core 的配置管理功能

pub mod config;
pub mod connection;
pub mod error;
pub mod manager;

// 重新导出主要组件
pub use config::DatabaseConfig;
pub use connection::{ConnectionStats, DatabaseConnection};
pub use error::{DatabaseError, DatabaseResult};
pub use manager::{DatabaseManager, HealthStatus};

// 便利函数
pub use manager::{
    // 用于 Axum AppState 的 Arc 包装版本
    create_connection_from_config,
    create_connection_from_url,
};
