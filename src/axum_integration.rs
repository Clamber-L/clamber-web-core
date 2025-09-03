//! Axum 集成示例
//!
//! 展示如何在 Axum 应用中使用数据库连接

use crate::database::{
    CreateUserRequest, DatabaseManager, UserDto, UserService, create_connection_from_url,
};
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use serde::Serialize;
use std::sync::Arc;

/// Axum 应用状态
#[derive(Clone)]
pub struct AppState {
    /// 数据库连接（可在处理器间共享）
    pub db: Arc<sea_orm::DatabaseConnection>,
}

impl AppState {
    /// 从数据库 URL 创建应用状态
    pub async fn from_url(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let db = create_connection_from_url(database_url).await?;
        Ok(Self { db })
    }

    /// 从 YAML 文件创建应用状态
    pub async fn from_yaml_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let manager = DatabaseManager::from_yaml_file(file_path).await?;
        Ok(Self {
            db: manager.get_connection(),
        })
    }

    /// 从 JSON 文件创建应用状态
    pub async fn from_json_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let manager = DatabaseManager::from_json_file(file_path).await?;
        Ok(Self {
            db: manager.get_connection(),
        })
    }

    /// 从环境变量创建应用状态
    pub async fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let manager = DatabaseManager::from_env().await?;
        Ok(Self {
            db: manager.get_connection(),
        })
    }
}

/// 创建 Axum 路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/users", post(create_user))
        .route("/users/:id", get(get_user))
}

/// 健康检查处理器
async fn health_check(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    // 测试数据库连接
    match state.db.ping().await {
        Ok(_) => Ok(Json(HealthResponse {
            status: "healthy".to_string(),
            database: "connected".to_string(),
        })),
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

/// 创建用户处理器
async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserDto>, StatusCode> {
    match UserService::create_user(&state.db, req).await {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 获取用户处理器
async fn get_user(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<UserDto>, StatusCode> {
    match UserService::find_by_id(&state.db, &id).await {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 健康检查响应
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    database: String,
}

/// 完整的应用构建器
pub struct WebApp {
    state: AppState,
}

impl WebApp {
    /// 从数据库 URL 创建应用
    pub async fn from_url(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let state = AppState::from_url(database_url).await?;
        Ok(Self { state })
    }

    /// 从配置文件创建应用
    pub async fn from_config_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let state = if file_path.ends_with(".yaml") || file_path.ends_with(".yml") {
            AppState::from_yaml_file(file_path).await?
        } else if file_path.ends_with(".json") {
            AppState::from_json_file(file_path).await?
        } else {
            return Err("不支持的配置文件格式，请使用 .yaml, .yml 或 .json".into());
        };

        Ok(Self { state })
    }

    /// 从环境变量创建应用
    pub async fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let state = AppState::from_env().await?;
        Ok(Self { state })
    }

    /// 创建 Axum 应用
    pub fn create_app(self) -> Router {
        create_routes().with_state(self.state)
    }

    /// 运行应用
    pub async fn run(self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let app = self.create_app();
        let listener = tokio::net::TcpListener::bind(addr).await?;

        tracing::info!("服务器启动在: {}", addr);
        axum::serve(listener, app).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            database: "connected".to_string(),
        };

        assert_eq!(response.status, "healthy");
        assert_eq!(response.database, "connected");
    }
}
