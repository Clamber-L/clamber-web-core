//! 数据库连接模块
//!
//! 提供 SeaORM 数据库连接的封装和扩展功能

use crate::database::{DatabaseConfig, DatabaseError, DatabaseResult};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing::{error, info, warn};

/// 数据库连接封装
#[derive(Debug, Clone)]
pub struct SeaOrmConnection {
    /// SeaORM 连接实例
    pub inner: DatabaseConnection,
    /// 配置信息
    config: DatabaseConfig,
}

impl SeaOrmConnection {
    /// 创建新的数据库连接
    pub async fn new(config: DatabaseConfig) -> DatabaseResult<Self> {
        // 验证配置
        config
            .validate()
            .map_err(|msg| DatabaseError::config(msg))?;

        info!("正在连接数据库: {}", mask_database_url(&config.url));

        // 创建连接选项
        let mut opt = ConnectOptions::new(&config.url);
        opt.max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect_timeout(config.connect_timeout())
            .acquire_timeout(config.acquire_timeout())
            .idle_timeout(config.idle_timeout())
            .max_lifetime(config.max_lifetime())
            .sqlx_logging(config.sql_logging);

        // 建立连接
        let connection = Database::connect(opt).await.map_err(|e| {
            error!("数据库连接失败: {}", e);
            DatabaseError::connection(format!("连接失败: {}", e))
        })?;

        info!("数据库连接成功建立");

        Ok(Self {
            inner: connection,
            config,
        })
    }

    /// 从数据库 URL 字符串创建管理器（最常用）
    pub async fn from_url(database_url: &str) -> DatabaseResult<Self> {
        info!("从 URL 创建数据库连接: {}", mask_database_url(database_url));
        let config = DatabaseConfig {
            url: database_url.to_string(),
            ..DatabaseConfig::default()
        };
        Self::new(config).await
    }

    /// 测试连接是否有效
    pub async fn ping(&self) -> DatabaseResult<()> {
        self.inner.ping().await.map_err(|e| {
            warn!("数据库连接测试失败: {}", e);
            DatabaseError::connection(format!("连接测试失败: {}", e))
        })?;

        info!("数据库连接测试成功");
        Ok(())
    }

    /// 关闭连接
    pub async fn close(self) -> DatabaseResult<()> {
        self.inner
            .close()
            .await
            .map_err(|e| DatabaseError::connection(format!("关闭连接失败: {}", e)))?;
        info!("数据库连接已关闭");
        Ok(())
    }

    /// 获取连接统计信息
    pub fn get_stats(&self) -> ConnectionStats {
        ConnectionStats {
            max_connections: self.config.max_connections,
            min_connections: self.config.min_connections,
            connect_timeout: self.config.connect_timeout_secs,
            acquire_timeout: self.config.acquire_timeout_secs,
        }
    }

}

/// 便利函数：从 URL 创建连接（最常用）
pub async fn create_connection_from_url(
    database_url: &str,
) -> DatabaseResult<DatabaseConnection> {
    let sea_connection = SeaOrmConnection::from_url(database_url).await?;
    Ok(sea_connection.inner)
}

/// 便利函数：从配置对象创建连接
pub async fn create_connection_from_config(
    config: DatabaseConfig,
) -> DatabaseResult<DatabaseConnection> {
    let sea_connection = SeaOrmConnection::new(config).await?;
    Ok(sea_connection.inner)
}

/// 连接统计信息
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub acquire_timeout: u64,
}

/// 数据库健康状态
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub response_time_ms: u64,
    pub message: String,
}

/// 屏蔽数据库 URL 中的敏感信息
pub fn mask_database_url(url: &str) -> String {
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
    fn test_mask_database_url() {
        let url = "mysql://user:password@localhost:3306/database";
        let masked = mask_database_url(url);
        assert!(masked.contains("***"));
        assert!(!masked.contains("password"));
    }

    #[test]
    fn test_connection_stats() {
        let config = DatabaseConfig::default();
        let stats = ConnectionStats {
            max_connections: config.max_connections,
            min_connections: config.min_connections,
            connect_timeout: config.connect_timeout_secs,
            acquire_timeout: config.acquire_timeout_secs,
        };

        assert_eq!(stats.max_connections, 100);
        assert_eq!(stats.min_connections, 5);
    }

    #[tokio::test]
    async fn test_invalid_config() {
        let mut config = DatabaseConfig::default();
        config.url = String::new(); // 无效的 URL

        let result = SeaOrmConnection::new(config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().is_config_error());
    }
}
