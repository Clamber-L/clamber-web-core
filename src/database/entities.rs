//! 示例实体模块
//!
//! 提供一些常用的实体模型示例，演示如何在 clamber-web-core 中使用 SeaORM

use sea_orm::Set;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 用户实体
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    /// 用户 ID
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,

    /// 用户名
    pub username: String,

    /// 邮箱
    pub email: String,

    /// 密码哈希
    pub password_hash: String,

    /// 用户角色
    pub role: String,

    /// 是否启用
    pub is_active: bool,

    /// 创建时间
    pub created_at: DateTimeUtc,

    /// 更新时间
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
    /// 插入前自动生成 ID 和时间戳
    fn new() -> Self {
        Self {
            id: Set(format!(
                "{}",
                chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
            )),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            is_active: Set(true),
            role: Set("user".to_string()),
            ..ActiveModelTrait::default()
        }
    }

    /// 更新前自动更新时间戳
    fn before_save<'life0, 'async_trait, C>(
        mut self,
        _db: &'life0 C,
        _insert: bool,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = Result<Self, DbErr>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        Self: 'async_trait,
        'life0: 'async_trait,
        C: ConnectionTrait + 'async_trait,
    {
        Box::pin(async move {
            self.updated_at = Set(chrono::Utc::now());
            Ok(self)
        })
    }
}

/// 用户数据传输对象
#[derive(Debug, Serialize, Deserialize)]
pub struct UserDto {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl From<Model> for UserDto {
    fn from(user: Model) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            is_active: user.is_active,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// 创建用户请求
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: Option<String>,
}

/// 用户服务
pub struct UserService;

impl UserService {
    /// 创建用户
    pub async fn create_user(
        db: &DatabaseConnection,
        req: CreateUserRequest,
    ) -> crate::database::DatabaseResult<UserDto> {
        // 简化的密码处理（生产环境中应使用正确的密码哈希）
        let password_hash = format!("hashed_{}", req.password);

        let user = ActiveModel {
            username: Set(req.username),
            email: Set(req.email),
            password_hash: Set(password_hash),
            role: Set(req.role.unwrap_or("user".to_string())),
            ..ActiveModel::new()
        };

        let user = user
            .insert(db)
            .await
            .map_err(crate::database::DatabaseError::from)?;

        Ok(user.into())
    }

    /// 根据 ID 查找用户
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: &str,
    ) -> crate::database::DatabaseResult<Option<UserDto>> {
        let user = Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(crate::database::DatabaseError::from)?;

        Ok(user.map(Into::into))
    }

    /// 删除用户
    pub async fn delete_user(
        db: &DatabaseConnection,
        id: &str,
    ) -> crate::database::DatabaseResult<bool> {
        let result = Entity::delete_by_id(id)
            .exec(db)
            .await
            .map_err(crate::database::DatabaseError::from)?;

        Ok(result.rows_affected > 0)
    }
}
