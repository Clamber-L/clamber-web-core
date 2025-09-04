# Clamber Web Core 使用说明

`clamber-web-core` 是基于 `clamber-core` 构建的 Web 基础库，提供数据库管理、Web 框架集成等核心功能。本文档将详细介绍如何使用该库。

## 📦 项目依赖

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
clamber-web-core = "0.1.0"
tokio = { version = "1.41.1", features = ["rt", "rt-multi-thread", "macros"] }
tracing-subscriber = "0.3"  # 可选：用于日志输出
```

## 🚀 快速开始

### 基本使用示例

```rust
use clamber_web_core::database::{create_connection_from_url, DatabaseConfig, SeaOrmConnection};
use tokio;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志（可选）
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // 方法1: 直接从 URL 创建连接（最简单）
    let db_url = "mysql://username:password@localhost:3306/database_name";
    let connection = create_connection_from_url(db_url).await?;
    
    // 测试连接
    connection.ping().await?;
    println!("数据库连接成功！");

    Ok(())
}
```

## 🔧 数据库配置

### DatabaseConfig 结构

```rust
use clamber_web_core::database::DatabaseConfig;

let config = DatabaseConfig {
    url: "mysql://username:password@localhost:3306/database_name".to_string(),
    max_connections: 100,        // 最大连接数
    min_connections: 5,          // 最小连接数
    connect_timeout_secs: 30,    // 连接超时（秒）
    acquire_timeout_secs: 30,    // 获取连接超时（秒）
    idle_timeout_secs: 600,      // 空闲超时（秒）
    max_lifetime_secs: 1800,     // 连接最大生命周期（秒）
    sql_logging: true,           // 是否启用SQL日志
    slow_threshold_ms: 1000,     // 慢查询阈值（毫秒）
};
```

### 使用默认配置

```rust
use clamber_web_core::database::DatabaseConfig;

// 使用默认配置，只需要提供数据库 URL
let mut config = DatabaseConfig::default();
config.url = "mysql://username:password@localhost:3306/database_name".to_string();
```

### 配置验证

```rust
// 验证配置有效性
match config.validate() {
    Ok(_) => println!("配置有效"),
    Err(e) => eprintln!("配置错误: {}", e),
}
```

## 🔗 连接管理

### 方式1: 使用便利函数（推荐）

```rust
use clamber_web_core::database::{create_connection_from_url, create_connection_from_config};

// 从 URL 创建
let connection = create_connection_from_url("mysql://user:pass@host:3306/db").await?;

// 从配置创建
let config = DatabaseConfig { /* 配置参数 */ };
let connection = create_connection_from_config(config).await?;
```

### 方式2: 使用 SeaOrmConnection

```rust
use clamber_web_core::database::{SeaOrmConnection, DatabaseConfig};

// 从配置创建
let config = DatabaseConfig {
    url: "mysql://user:pass@host:3306/db".to_string(),
    max_connections: 50,
    ..DatabaseConfig::default()
};

let db_conn = SeaOrmConnection::new(config).await?;

// 或者直接从 URL 创建
let db_conn = SeaOrmConnection::from_url("mysql://user:pass@host:3306/db").await?;

// 使用连接
db_conn.ping().await?;

// 获取底层 SeaORM 连接
let sea_connection = &db_conn.inner;

// 获取连接统计信息
let stats = db_conn.get_stats();
println!("最大连接数: {}", stats.max_connections);
```

### 连接测试和健康检查

```rust
// 测试连接是否有效
match db_conn.ping().await {
    Ok(_) => println!("连接正常"),
    Err(e) => eprintln!("连接异常: {}", e),
}

// 安全关闭连接
db_conn.close().await?;
```

## 📊 实际使用场景

### 场景1: 简单的数据库操作

```rust
use clamber_web_core::database::create_connection_from_url;
use sea_orm::{EntityTrait, Set, ActiveModelTrait};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建连接
    let db = create_connection_from_url("mysql://user:pass@localhost:3306/mydb").await?;
    
    // 使用 SeaORM 进行数据库操作
    // 这里假设您已经定义了实体模型
    // let user = user::ActiveModel {
    //     name: Set("张三".to_owned()),
    //     email: Set("zhangsan@example.com".to_owned()),
    //     ..Default::default()
    // };
    // let user = user.insert(&db).await?;
    
    Ok(())
}
```

### 场景2: Web 应用中使用

```rust
use axum::{
    extract::State,
    routing::get,
    Router,
    Json,
};
use clamber_web_core::database::create_connection_from_url;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

// 应用状态
#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
}

// 路由处理函数
async fn health_check(State(state): State<AppState>) -> Json<&'static str> {
    match state.db.ping().await {
        Ok(_) => Json("数据库连接正常"),
        Err(_) => Json("数据库连接异常"),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建数据库连接
    let db = create_connection_from_url("mysql://user:pass@localhost:3306/mydb").await?;
    
    // 创建应用状态
    let app_state = AppState { db };
    
    // 创建路由
    let app = Router::new()
        .route("/health", get(health_check))
        .with_state(app_state);
    
    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("服务器启动在 http://0.0.0.0:3000");
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

### 场景3: 批量连接测试

```rust
use clamber_web_core::database::create_connection_from_url;
use tokio::time::{Duration, Instant};
use tracing::{info, warn};

async fn test_multiple_connections() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = "mysql://user:pass@localhost:3306/mydb";
    
    // 并发创建多个连接
    let mut handles = vec![];
    
    for i in 0..5 {
        let url = database_url.to_string();
        let handle = tokio::spawn(async move {
            let start = Instant::now();
            
            match create_connection_from_url(&url).await {
                Ok(conn) => {
                    let connect_time = start.elapsed();
                    
                    // 测试 ping 响应时间
                    let ping_start = Instant::now();
                    match conn.ping().await {
                        Ok(_) => {
                            let ping_time = ping_start.elapsed();
                            info!("连接 {} - 建立时间: {:?}, Ping时间: {:?}", 
                                  i + 1, connect_time, ping_time);
                            Ok(())
                        }
                        Err(e) => {
                            warn!("连接 {} ping 失败: {}", i + 1, e);
                            Err(e)
                        }
                    }
                }
                Err(e) => {
                    warn!("连接 {} 建立失败: {}", i + 1, e);
                    Err(e)
                }
            }
        });
        
        handles.push(handle);
    }
    
    // 等待所有连接完成
    for (i, handle) in handles.into_iter().enumerate() {
        match handle.await? {
            Ok(_) => info!("✅ 连接 {} 测试成功", i + 1),
            Err(e) => warn!("❌ 连接 {} 测试失败: {}", i + 1, e),
        }
    }
    
    Ok(())
}
```

## 🚨 错误处理

### 错误类型

```rust
use clamber_web_core::database::{DatabaseError, DatabaseResult};

// 函数返回类型
async fn database_operation() -> DatabaseResult<()> {
    // 数据库操作...
    Ok(())
}

// 错误处理示例
match database_operation().await {
    Ok(_) => println!("操作成功"),
    Err(DatabaseError::Connection { message }) => {
        eprintln!("连接错误: {}", message);
    }
    Err(DatabaseError::Config { message }) => {
        eprintln!("配置错误: {}", message);
    }
    Err(DatabaseError::EntityNotFound { entity_name, id }) => {
        eprintln!("实体不存在: {} (ID: {})", entity_name, id);
    }
    Err(e) => {
        eprintln!("其他错误: {}", e);
    }
}
```

### 错误判断方法

```rust
if let Err(e) = result {
    if e.is_connection_error() {
        // 处理连接错误
        eprintln!("连接问题，请检查网络和数据库状态");
    } else if e.is_config_error() {
        // 处理配置错误
        eprintln!("配置问题，请检查数据库配置");
    } else if e.is_not_found_error() {
        // 处理实体不存在错误
        eprintln!("请求的数据不存在");
    }
}
```

## 🔒 安全注意事项

### URL 敏感信息屏蔽

```rust
use clamber_web_core::database::connection::mask_database_url;

let url = "mysql://user:password@localhost:3306/database";
let masked = mask_database_url(&url);
println!("安全的URL显示: {}", masked); // 输出: mysql://***:***@localhost:3306/database
```

### 生产环境配置建议

```rust
// 推荐的生产环境配置
let config = DatabaseConfig {
    url: std::env::var("DATABASE_URL")
        .expect("DATABASE_URL 环境变量必须设置"),
    max_connections: 100,
    min_connections: 10,
    connect_timeout_secs: 30,
    acquire_timeout_secs: 10,
    idle_timeout_secs: 300,
    max_lifetime_secs: 1800,
    sql_logging: false,  // 生产环境建议关闭
    slow_threshold_ms: 500,
};
```

## 📝 完整示例参考

项目中的 `examples/test_db.rs` 文件包含了完整的测试用例，展示了各种使用场景：

1. 基本连接测试
2. SeaOrmConnection 结构体功能测试
3. 便利函数使用测试
4. 连接性能测试
5. 并发连接测试
6. 错误处理测试

运行示例：

```bash
cargo run --example test_db
```

## 🛠️ 常见问题

### Q: 连接超时怎么办？
A: 检查网络连接，调整 `connect_timeout_secs` 参数，确保数据库服务正常运行。

### Q: 如何优化连接池性能？
A: 根据应用负载调整 `max_connections` 和 `min_connections`，监控连接使用情况。

### Q: 如何在多个模块中共享连接？
A: 使用 `Arc<DatabaseConnection>` 或将连接放在应用状态中。

### Q: 支持哪些数据库？
A: 目前支持 MySQL，通过 SeaORM 可以扩展支持 PostgreSQL、SQLite 等。

## 📚 相关资源

- [SeaORM 官方文档](https://www.sea-ql.org/SeaORM/)
- [Axum 官方文档](https://docs.rs/axum/)
- [Clamber Core 文档](https://docs.rs/clamber-core/)

---

如果您在使用过程中遇到问题或有改进建议，欢迎提交 Issue 或 Pull Request！