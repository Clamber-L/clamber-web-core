//! 数据库模块
//!
//! 提供基于 SeaORM 的数据库连接管理、配置和工具函数
//! 集成 clamber-core 的配置管理功能

pub mod config;
pub mod connection;
pub mod entities;
pub mod error;
pub mod manager;

// 重新导出主要组件
pub use config::DatabaseConfig;
pub use connection::{ConnectionStats, DatabaseConnection};
pub use entities::{
    CreateUserRequest, Entity as UserEntity, Model as UserModel, UserDto, UserService,
};
pub use error::{DatabaseError, DatabaseResult};
pub use manager::{DatabaseManager, HealthStatus};

// 便利函数
pub use manager::{
    create_connection,
    create_connection_from_config,
    create_connection_from_env,
    create_connection_from_json,
    // 新增的便利函数
    create_connection_from_url,
    create_connection_from_yaml,
    create_connection_with_config,
    get_global_connection,
    get_global_database,
    init_global_database,
    init_global_database_from_config,
    init_global_database_from_env,
};
