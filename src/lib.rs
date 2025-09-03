//! Clamber Web Core
//!
//! 基于 clamber-core 构建的 Web 基础 crate，提供：
//! - 数据库管理（基于 SeaORM）
//! - Web 框架集成（基于 Axum）
//! - 认证和授权
//! - 统一错误处理
//! - 配置管理

pub mod database;

// 重新导出主要模块
pub use database::*;

// 重新导出核心依赖
pub use axum;
pub use chrono;
pub use clamber_core;
pub use sea_orm;
pub use serde;
pub use serde_json;
pub use tokio;
