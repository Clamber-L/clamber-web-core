//! 数据库管理器模块
//!
//! 提供多种方式创建数据库连接，专为 Axum AppState 设计

use crate::database::{DatabaseConfig, DatabaseConnection, DatabaseError, DatabaseResult};
use std::sync::Arc;
use tracing::info;

/// 数据库管理器 - 专为 Axum AppState 设计
pub struct DatabaseManager {
    connection: sea_orm::DatabaseConnection,
}

impl DatabaseManager {
    /// 从配置创建数据库管理器
    pub async fn new(config: DatabaseConfig) -> DatabaseResult<Self> {
        let db_conn = DatabaseConnection::new(config).await?;
        Ok(Self {
            connection: db_conn.get_connection().clone(),
        })
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

    /// 从 YAML 配置文件创建管理器
    pub async fn from_yaml_file(file_path: &str) -> DatabaseResult<Self> {
        info!("从 YAML 文件创建数据库连接: {}", file_path);
        let yaml_content = tokio::fs::read_to_string(file_path)
            .await
            .map_err(|e| DatabaseError::config(format!("读取 YAML 文件失败: {}", e)))?;

        let config: DatabaseConfig = serde_yaml::from_str(&yaml_content)
            .map_err(|e| DatabaseError::config(format!("解析 YAML 配置失败: {}", e)))?;

        Self::new(config).await
    }

    /// 从 JSON 配置文件创建管理器
    pub async fn from_json_file(file_path: &str) -> DatabaseResult<Self> {
        info!("从 JSON 文件创建数据库连接: {}", file_path);
        let json_content = tokio::fs::read_to_string(file_path)
            .await
            .map_err(|e| DatabaseError::config(format!("读取 JSON 文件失败: {}", e)))?;

        let config: DatabaseConfig = serde_json::from_str(&json_content)
            .map_err(|e| DatabaseError::config(format!("解析 JSON 配置失败: {}", e)))?;

        Self::new(config).await
    }

    /// 从环境变量创建管理器
    pub async fn from_env() -> DatabaseResult<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| DatabaseError::config("DATABASE_URL 环境变量未设置"))?;

        Self::from_url(&database_url).await
    }

    /// 获取 SeaORM 连接（用于 Axum AppState）
    pub fn get_connection(&self) -> Arc<sea_orm::DatabaseConnection> {
        Arc::new(self.connection.clone())
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
) -> DatabaseResult<Arc<sea_orm::DatabaseConnection>> {
    let manager = DatabaseManager::from_url(database_url).await?;
    Ok(manager.get_connection())
}

/// 便利函数：从 YAML 文件创建连接
pub async fn create_connection_from_yaml(
    file_path: &str,
) -> DatabaseResult<Arc<sea_orm::DatabaseConnection>> {
    let manager = DatabaseManager::from_yaml_file(file_path).await?;
    Ok(manager.get_connection())
}

/// 便利函数：从 JSON 文件创建连接
pub async fn create_connection_from_json(
    file_path: &str,
) -> DatabaseResult<Arc<sea_orm::DatabaseConnection>> {
    let manager = DatabaseManager::from_json_file(file_path).await?;
    Ok(manager.get_connection())
}

/// 便利函数：从环境变量创建连接
pub async fn create_connection_from_env() -> DatabaseResult<Arc<sea_orm::DatabaseConnection>> {
    let manager = DatabaseManager::from_env().await?;
    Ok(manager.get_connection())
}

/// 便利函数：从配置对象创建连接
pub async fn create_connection_from_config(
    config: DatabaseConfig,
) -> DatabaseResult<Arc<sea_orm::DatabaseConnection>> {
    let manager = DatabaseManager::new(config).await?;
    Ok(manager.get_connection())
}

// 保留原有的函数以保持兼容性
pub async fn create_connection(config: DatabaseConfig) -> DatabaseResult<DatabaseConnection> {
    DatabaseConnection::new(config).await
}

pub async fn create_connection_with_config() -> DatabaseResult<DatabaseConnection> {
    let config = DatabaseConfig::default();
    DatabaseConnection::new(config).await
}

// 简化的全局函数（占位符）
pub async fn init_global_database(_config: DatabaseConfig) -> DatabaseResult<()> {
    Ok(())
}

pub async fn init_global_database_from_env() -> DatabaseResult<()> {
    Ok(())
}

pub async fn init_global_database_from_config() -> DatabaseResult<()> {
    Ok(())
}

pub fn get_global_database() -> DatabaseResult<Arc<DatabaseManager>> {
    Err(DatabaseError::config("全局数据库管理器未实现"))
}

pub fn get_global_connection() -> DatabaseResult<Arc<DatabaseManager>> {
    get_global_database()
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
