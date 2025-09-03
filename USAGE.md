# clamber-web-core 使用指南

## 数据库连接创建

clamber-web-core 提供了多种灵活的方式来创建数据库连接，特别适合 Axum AppState 的使用场景。

### 1. 从 URL 字符串创建（最常用）

```rust
use clamber_web_core::{create_connection_from_url, AppState, WebApp};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 直接从 URL 创建连接
    let db_url = "mysql://user:password@localhost:3306/mydb";
    let db_connection = create_connection_from_url(db_url).await?;
    
    // 或者创建完整的 AppState
    let app_state = AppState::from_url(db_url).await?;
    
    // 创建完整的 Web 应用
    let web_app = WebApp::from_url(db_url).await?;
    web_app.run("0.0.0.0:3000").await?;
    
    Ok(())
}
```

### 2. 从 YAML 配置文件创建

**config.yaml:**
```yaml
url: "mysql://user:password@localhost:3306/mydb"
max_connections: 100
min_connections: 5
connect_timeout_secs: 30
acquire_timeout_secs: 30
idle_timeout_secs: 600
max_lifetime_secs: 1800
sql_logging: true
slow_threshold_ms: 1000
```

**Rust 代码:**
```rust
use clamber_web_core::{create_connection_from_yaml, WebApp};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从 YAML 文件创建连接
    let db_connection = create_connection_from_yaml("config.yaml").await?;
    
    // 或者创建完整的 Web 应用
    let web_app = WebApp::from_config_file("config.yaml").await?;
    web_app.run("0.0.0.0:3000").await?;
    
    Ok(())
}
```

### 3. 从 JSON 配置文件创建

**config.json:**
```json
{
  "url": "mysql://user:password@localhost:3306/mydb",
  "max_connections": 100,
  "min_connections": 5,
  "connect_timeout_secs": 30,
  "acquire_timeout_secs": 30,
  "idle_timeout_secs": 600,
  "max_lifetime_secs": 1800,
  "sql_logging": true,
  "slow_threshold_ms": 1000
}
```

**Rust 代码:**
```rust
use clamber_web_core::{create_connection_from_json, WebApp};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从 JSON 文件创建连接
    let db_connection = create_connection_from_json("config.json").await?;
    
    // 或者创建完整的 Web 应用
    let web_app = WebApp::from_config_file("config.json").await?;
    web_app.run("0.0.0.0:3000").await?;
    
    Ok(())
}
```

### 4. 从环境变量创建

```bash
export DATABASE_URL="mysql://user:password@localhost:3306/mydb"
```

```rust
use clamber_web_core::{create_connection_from_env, WebApp};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量创建连接
    let db_connection = create_connection_from_env().await?;
    
    // 或者创建完整的 Web 应用
    let web_app = WebApp::from_env().await?;
    web_app.run("0.0.0.0:3000").await?;
    
    Ok(())
}
```

## Axum 集成示例

### 简单的 Axum 应用

```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use clamber_web_core::{AppState, UserService, CreateUserRequest, UserDto};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::init();
    
    // 创建应用状态
    let app_state = AppState::from_url("mysql://user:password@localhost:3306/mydb").await?;
    
    // 创建路由
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/users", post(create_user))
        .route("/users/:id", get(get_user))
        .with_state(app_state);
    
    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("服务器启动在: http://0.0.0.0:3000");
    axum::serve(listener, app).await?;
    
    Ok(())
}

// 健康检查
async fn health_check(State(state): State<AppState>) -> Result<&'static str, StatusCode> {
    use sea_orm::ConnectionTrait;
    
    match state.db.ping().await {
        Ok(_) => Ok("数据库连接正常"),
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

// 创建用户
async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserDto>, StatusCode> {
    match UserService::create_user(&state.db, req).await {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// 获取用户
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<UserDto>, StatusCode> {
    match UserService::find_by_id(&state.db, &id).await {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
```

### 使用 WebApp 构建器

```rust
use clamber_web_core::WebApp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::init();
    
    // 方式1：从 URL 创建
    let app = WebApp::from_url("mysql://user:password@localhost:3306/mydb").await?;
    
    // 方式2：从配置文件创建
    // let app = WebApp::from_config_file("config.yaml").await?;
    
    // 方式3：从环境变量创建
    // let app = WebApp::from_env().await?;
    
    // 启动应用
    app.run("0.0.0.0:3000").await?;
    
    Ok(())
}
```

## 配置选项

### DatabaseConfig 完整配置

```rust
use clamber_web_core::{DatabaseConfig, create_connection_from_config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DatabaseConfig {
        url: "mysql://user:password@localhost:3306/mydb".to_string(),
        max_connections: 100,        // 最大连接数
        min_connections: 5,          // 最小连接数
        connect_timeout_secs: 30,    // 连接超时（秒）
        acquire_timeout_secs: 30,    // 获取连接超时（秒）
        idle_timeout_secs: 600,      // 空闲超时（秒）
        max_lifetime_secs: 1800,     // 连接最大生命周期（秒）
        sql_logging: true,           // 是否启用SQL日志
        slow_threshold_ms: 1000,     // 慢查询阈值（毫秒）
    };
    
    let db_connection = create_connection_from_config(config).await?;
    
    Ok(())
}
```

## 最佳实践

1. **生产环境使用环境变量**：避免在代码中硬编码数据库连接信息
2. **配置连接池**：根据应用负载调整 `max_connections` 和 `min_connections`
3. **监控慢查询**：启用 `sql_logging` 并设置合适的 `slow_threshold_ms`
4. **健康检查**：定期使用 `ping()` 方法检查数据库连接状态
5. **错误处理**：总是处理数据库操作可能产生的错误

这个设计使得在 Axum 中使用数据库连接变得非常简单和灵活，无论您是从 URL、配置文件还是环境变量创建连接，都能轻松集成到 AppState 中。