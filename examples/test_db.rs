//! 数据库连接测试用例
//!
//! 测试 clamber-web-core 数据库模块的各种功能

use clamber_web_core::database::{DatabaseConfig, SeaOrmConnection, create_connection_from_url};
use std::time::Duration;
use tokio::time::Instant;
use tracing::{error, info, warn};

// 数据库连接配置
const DB_HOST: &str = "127.0.0.1";
const DB_USERNAME: &str = "root";
const DB_PASSWORD: &str = "lsw0516";
const DB_PORT: u16 = 3306;
const DB_NAME: &str = "clamber"; // 默认数据库名，可以根据实际情况修改

/// 构建数据库连接 URL
fn build_database_url() -> String {
    format!(
        "mysql://{}:{}@{}:{}/{}",
        DB_USERNAME, DB_PASSWORD, DB_HOST, DB_PORT, DB_NAME
    )
}

/// 测试 1: 基本连接测试
async fn test_basic_connection() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试 1: 基本数据库连接");

    let database_url = build_database_url();
    let connection = create_connection_from_url(&database_url).await?;

    // 测试连接
    connection.ping().await?;
    info!("✅ 基本连接测试成功");

    Ok(())
}

/// 测试 2: SeaOrmConnection 结构体测试
async fn test_database_connection_struct() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试 2: SeaOrmConnection 结构体功能");

    let config = DatabaseConfig {
        url: build_database_url(),
        max_connections: 10,
        min_connections: 2,
        connect_timeout_secs: 10,
        acquire_timeout_secs: 5,
        idle_timeout_secs: 300,
        max_lifetime_secs: 3600,
        sql_logging: true,
        slow_threshold_ms: 1000,
    };

    let db_conn = SeaOrmConnection::new(config.clone()).await?;

    // 测试 ping
    db_conn.ping().await?;
    info!("✅ SeaOrmConnection ping 测试成功");

    // 测试获取连接引用（通过 inner 字段）
    let conn_ref = &db_conn.inner;
    conn_ref.ping().await?;
    info!("✅ 获取连接引用测试成功");

    // 测试连接统计信息
    let stats = SeaOrmConnection::new(config).await?.get_stats();
    info!(
        "📊 连接统计: 最大连接数={}, 最小连接数={}",
        stats.max_connections, stats.min_connections
    );

    Ok(())
}

/// 测试 4: 便利函数测试
async fn test_convenience_functions() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试 4: 便利函数");

    let database_url = build_database_url();

    // 测试直接从 URL 创建连接
    let conn1 = create_connection_from_url(&database_url).await?;
    conn1.ping().await?;
    info!("✅ create_connection_from_url 测试成功");

    Ok(())
}

/// 测试 5: 连接性能测试
async fn test_connection_performance() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试 5: 连接性能测试");

    let database_url = build_database_url();

    // 测试连接建立时间
    let start = Instant::now();
    let connection = create_connection_from_url(&database_url).await?;
    let connection_time = start.elapsed();

    // 测试 ping 响应时间
    let start = Instant::now();
    connection.ping().await?;
    let ping_time = start.elapsed();

    info!("⏱️ 连接建立时间: {:?}", connection_time);
    info!("⏱️ Ping 响应时间: {:?}", ping_time);

    if ping_time < Duration::from_millis(1000) {
        info!("✅ 连接性能良好 (< 1秒)");
    } else {
        warn!("⚠️ 连接响应较慢 (> 1秒)");
    }

    Ok(())
}

/// 测试 6: 并发连接测试
async fn test_concurrent_connections() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试 6: 并发连接测试");

    let database_url = build_database_url();
    let mut handles = vec![];

    // 创建 5 个并发连接
    for i in 0..5 {
        let url = database_url.clone();
        let handle = tokio::spawn(async move {
            let connection = create_connection_from_url(&url)
                .await
                .map_err(|e| format!("连接失败: {}", e))?;
            connection
                .ping()
                .await
                .map_err(|e| format!("ping失败: {}", e))?;
            info!("✅ 并发连接 {} 成功", i + 1);
            Ok::<(), String>(())
        });
        handles.push(handle);
    }

    // 等待所有连接完成
    for handle in handles {
        let _ = handle.await.map_err(|e| format!("任务执行失败: {}", e))??;
    }

    info!("✅ 并发连接测试完成");
    Ok(())
}

/// 测试 7: 错误处理测试
async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试 7: 错误处理测试");

    // 测试无效的连接 URL
    let invalid_url = "mysql://invalid:invalid@invalid:3306/invalid";

    match create_connection_from_url(invalid_url).await {
        Ok(_) => {
            error!("❌ 预期连接失败，但连接成功了");
            return Err("连接到无效地址应该失败".into());
        }
        Err(e) => {
            info!("✅ 正确处理了无效连接: {}", e);
        }
    }

    // 测试无效配置
    let invalid_config = DatabaseConfig {
        url: String::new(), // 空 URL
        ..DatabaseConfig::default()
    };

    match SeaOrmConnection::new(invalid_config).await {
        Ok(_) => {
            error!("❌ 预期配置验证失败，但成功了");
            return Err("空 URL 配置应该失败".into());
        }
        Err(e) => {
            info!("✅ 正确处理了无效配置: {}", e);
        }
    }

    info!("✅ 错误处理测试完成");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 开始数据库连接测试");
    info!("📡 连接目标: {}", DB_HOST);

    // 运行所有测试
    type TestFn = fn() -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send>,
    >;

    let tests: Vec<(&str, TestFn)> = vec![
        ("基本连接测试", || Box::pin(test_basic_connection())),
        ("SeaOrmConnection 测试", || {
            Box::pin(test_database_connection_struct())
        }),
        ("便利函数测试", || {
            Box::pin(test_convenience_functions())
        }),
        ("连接性能测试", || {
            Box::pin(test_connection_performance())
        }),
        ("并发连接测试", || {
            Box::pin(test_concurrent_connections())
        }),
        ("错误处理测试", || Box::pin(test_error_handling())),
    ];

    let mut passed = 0;
    let total = tests.len();

    for (name, test_fn) in tests {
        info!("\n{}", "=".repeat(50));
        match test_fn().await {
            Ok(_) => {
                info!("✅ {} 通过", name);
                passed += 1;
            }
            Err(e) => {
                error!("❌ {} 失败: {}", name, e);
            }
        }
    }

    info!("\n{}", "=".repeat(50));
    info!("🏁 测试完成: {}/{} 通过", passed, total);

    if passed == total {
        info!("🎉 所有测试通过！");
    } else {
        warn!("⚠️ 部分测试失败，请检查连接配置和网络状态");
    }

    Ok(())
}
