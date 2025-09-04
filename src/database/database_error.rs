//! 数据库错误处理模块
//!
//! 定义数据库相关的错误类型，集成 clamber-core 的错误处理系统

use thiserror::Error;

/// 数据库相关错误类型
#[derive(Error, Debug)]
pub enum DatabaseError {
    /// SeaORM 数据库错误
    #[error("数据库操作错误: {0}")]
    SeaOrm(#[from] sea_orm::DbErr),

    /// 连接错误
    #[error("数据库连接错误: {message}")]
    Connection { message: String },

    /// 配置错误
    #[error("数据库配置错误: {message}")]
    Config { message: String },

    /// 迁移错误
    #[error("数据库迁移错误: {message}")]
    Migration { message: String },

    /// 事务错误
    #[error("数据库事务错误: {message}")]
    Transaction { message: String },

    /// 查询错误
    #[error("查询错误: {message}")]
    Query { message: String },

    /// 实体不存在错误
    #[error("实体不存在: {entity_name} with id: {id}")]
    EntityNotFound { entity_name: String, id: String },

    /// 约束违反错误
    #[error("约束违反: {constraint}")]
    ConstraintViolation { constraint: String },

    /// 核心库错误
    #[error("核心库错误: {0}")]
    Core(#[from] clamber_core::ClamberError),
}

impl DatabaseError {
    /// 创建连接错误
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
        }
    }

    /// 创建配置错误
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// 创建迁移错误
    pub fn migration(message: impl Into<String>) -> Self {
        Self::Migration {
            message: message.into(),
        }
    }

    /// 创建事务错误
    pub fn transaction(message: impl Into<String>) -> Self {
        Self::Transaction {
            message: message.into(),
        }
    }

    /// 创建查询错误
    pub fn query(message: impl Into<String>) -> Self {
        Self::Query {
            message: message.into(),
        }
    }

    /// 创建实体不存在错误
    pub fn entity_not_found(entity_name: impl Into<String>, id: impl Into<String>) -> Self {
        Self::EntityNotFound {
            entity_name: entity_name.into(),
            id: id.into(),
        }
    }

    /// 创建约束违反错误
    pub fn constraint_violation(constraint: impl Into<String>) -> Self {
        Self::ConstraintViolation {
            constraint: constraint.into(),
        }
    }

    /// 判断是否为连接错误
    pub fn is_connection_error(&self) -> bool {
        matches!(
            self,
            DatabaseError::Connection { .. } | DatabaseError::SeaOrm(sea_orm::DbErr::Conn(_))
        )
    }

    /// 判断是否为配置错误
    pub fn is_config_error(&self) -> bool {
        matches!(self, DatabaseError::Config { .. })
    }

    /// 判断是否为约束违反错误
    pub fn is_constraint_error(&self) -> bool {
        matches!(self, DatabaseError::ConstraintViolation { .. })
    }

    /// 判断是否为实体不存在错误
    pub fn is_not_found_error(&self) -> bool {
        matches!(self, DatabaseError::EntityNotFound { .. })
    }
}

/// 数据库操作结果类型
pub type DatabaseResult<T> = Result<T, DatabaseError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = DatabaseError::connection("连接失败");
        assert!(error.is_connection_error());
        assert_eq!(error.to_string(), "数据库连接错误: 连接失败");
    }

    #[test]
    fn test_entity_not_found() {
        let error = DatabaseError::entity_not_found("User", "123");
        assert!(error.is_not_found_error());
        assert_eq!(error.to_string(), "实体不存在: User with id: 123");
    }

    #[test]
    fn test_constraint_violation() {
        let error = DatabaseError::constraint_violation("unique_email");
        assert!(error.is_constraint_error());
        assert_eq!(error.to_string(), "约束违反: unique_email");
    }
}
