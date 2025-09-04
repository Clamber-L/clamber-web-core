//! Redis 错误处理模块
//!
//! 定义 Redis 相关的错误类型，集成 clamber-core 的错误处理系统

use thiserror::Error;

/// Redis 相关错误类型
#[derive(Error, Debug)]
pub enum RedisError {
    /// Redis 库错误
    #[error("Redis 操作错误: {0}")]
    Redis(#[from] redis::RedisError),

    /// 连接错误
    #[error("Redis 连接错误: {message}")]
    Connection { message: String },

    /// 配置错误
    #[error("Redis 配置错误: {message}")]
    Config { message: String },

    /// 连接池错误
    #[error("Redis 连接池错误: {message}")]
    Pool { message: String },

    /// 序列化错误
    #[error("序列化错误: {message}")]
    Serialization { message: String },

    /// 反序列化错误
    #[error("反序列化错误: {message}")]
    Deserialization { message: String },

    /// 键不存在错误
    #[error("键不存在: {key}")]
    KeyNotFound { key: String },

    /// 类型转换错误
    #[error("类型转换错误: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    /// 超时错误
    #[error("操作超时: {operation}")]
    Timeout { operation: String },

    /// 核心库错误
    #[error("核心库错误: {0}")]
    Core(#[from] clamber_core::ClamberError),
}

impl RedisError {
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

    /// 创建连接池错误
    pub fn pool(message: impl Into<String>) -> Self {
        Self::Pool {
            message: message.into(),
        }
    }

    /// 创建序列化错误
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::Serialization {
            message: message.into(),
        }
    }

    /// 创建反序列化错误
    pub fn deserialization(message: impl Into<String>) -> Self {
        Self::Deserialization {
            message: message.into(),
        }
    }

    /// 创建键不存在错误
    pub fn key_not_found(key: impl Into<String>) -> Self {
        Self::KeyNotFound { key: key.into() }
    }

    /// 创建类型转换错误
    pub fn type_mismatch(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::TypeMismatch {
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// 创建超时错误
    pub fn timeout(operation: impl Into<String>) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }

    /// 判断是否为连接错误
    pub fn is_connection_error(&self) -> bool {
        matches!(self, RedisError::Connection { .. } | RedisError::Redis(_))
    }

    /// 判断是否为配置错误
    pub fn is_config_error(&self) -> bool {
        matches!(self, RedisError::Config { .. })
    }

    /// 判断是否为连接池错误
    pub fn is_pool_error(&self) -> bool {
        matches!(self, RedisError::Pool { .. })
    }

    /// 判断是否为序列化相关错误
    pub fn is_serialization_error(&self) -> bool {
        matches!(
            self,
            RedisError::Serialization { .. } | RedisError::Deserialization { .. }
        )
    }

    /// 判断是否为键不存在错误
    pub fn is_not_found_error(&self) -> bool {
        matches!(self, RedisError::KeyNotFound { .. })
    }

    /// 判断是否为超时错误
    pub fn is_timeout_error(&self) -> bool {
        matches!(self, RedisError::Timeout { .. })
    }
}

/// Redis 操作结果类型
pub type RedisResult<T> = Result<T, RedisError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = RedisError::connection("连接失败");
        assert!(error.is_connection_error());
        assert_eq!(error.to_string(), "Redis 连接错误: 连接失败");
    }

    #[test]
    fn test_key_not_found() {
        let error = RedisError::key_not_found("user:123");
        assert!(error.is_not_found_error());
        assert_eq!(error.to_string(), "键不存在: user:123");
    }

    #[test]
    fn test_type_mismatch() {
        let error = RedisError::type_mismatch("string", "hash");
        assert_eq!(error.to_string(), "类型转换错误: expected string, got hash");
    }

    #[test]
    fn test_timeout() {
        let error = RedisError::timeout("GET operation");
        assert!(error.is_timeout_error());
        assert_eq!(error.to_string(), "操作超时: GET operation");
    }

    #[test]
    fn test_serialization_error() {
        let error = RedisError::serialization("JSON parsing failed");
        assert!(error.is_serialization_error());
        assert_eq!(error.to_string(), "序列化错误: JSON parsing failed");
    }
}
