//! 数据库管理器模块
//!
//! 提供多种方式创建数据库连接，专为 Axum AppState 设计

use crate::database::{DatabaseConfig, DatabaseConnection, DatabaseError, DatabaseResult};
use tracing::info;

/// 数据库管理器 - 专为 Axum AppState 设计
pub struct DatabaseManager {
    connection: sea_orm::DatabaseConnection,
}

impl DatabaseManager {
    /// 从配置创建数据库管理器
    pub async fn new(config: DatabaseConfig) -> DatabaseResult<Self> {
        let conn = DatabaseConnection::new(config).await?;
        Ok(Self {
            connection: conn.inner,
        })
    }

    /// 获取数据库连接引用
    pub fn get_connection(&self) -> &sea_orm::DatabaseConnection {
        &self.connection
    }

    /// 从数据库 URL 字符串创建管理器（最常用）
    pub async fn from_url(database_url: &str) -> DatabaseResult<Self> {
        info!("从 URL 创建数据库连接: {}", mask_url(database_url));
        let config = DatabaseConfig {
            url: database_url.to_string(),
            ..DatabaseConfig::default()
        };
        Self::new(config).await
    }

    /// 测试连接是否有效
    pub async fn ping(&self) -> DatabaseResult<()> {
        self.connection
            .ping()
            .await
            .map_err(|e| DatabaseError::connection(format!("连接测试失败: {}", e)))?;
        Ok(())
    }
}

/// 便利函数：从 URL 创建连接（最常用）
pub async fn create_connection_from_url(
    database_url: &str,
) -> DatabaseResult<sea_orm::DatabaseConnection> {
    let manager = DatabaseManager::from_url(database_url).await?;
    Ok(manager.connection)
}

/// 便利函数：从配置对象创建连接
pub async fn create_connection_from_config(
    config: DatabaseConfig,
) -> DatabaseResult<sea_orm::DatabaseConnection> {
    let manager = DatabaseManager::new(config).await?;
    Ok(manager.connection)
}

/// 数据库健康状态
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub response_time_ms: u64,
    pub message: String,
}

/// 屏蔽 URL 中的敏感信息
fn mask_url(url: &str) -> String {
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
