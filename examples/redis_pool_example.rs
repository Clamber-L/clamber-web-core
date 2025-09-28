//! Redis 连接池使用示例
//!
//! 展示如何使用 clamber-web-core 的 Redis 连接池功能

use clamber_web_core::redis::{RedisConfig, RedisConnection, create_redis_connection_from_url};
use std::future::Future;
use std::time::Instant;
use tokio::time::sleep;
use tracing::{info, warn};

/// 示例1: 基本连接池使用
async fn example_basic_pool_usage() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 示例1: 基本连接池使用");

    let redis_url = "redis://localhost:6379";

    // 创建连接（内部使用连接池）
    let mut connection = create_redis_connection_from_url(redis_url).await?;

    // 测试连接
    connection.ping().await?;
    info!("✅ 连接池连接成功");

    // 获取连接池统计信息
    let stats = connection.get_pool_stats();
    info!(
        "📊 连接池统计: 最大连接数={}, 最小连接数={}",
        stats.max_connections, stats.min_connections
    );

    Ok(())
}

/// 示例2: 并发使用连接池
async fn example_concurrent_pool_usage() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 示例2: 并发使用连接池");

    let redis_url = "redis://localhost:6379";
    let mut handles = vec![];

    // 创建多个并发任务，共享连接池
    for i in 0..5 {
        let url = redis_url.to_string();
        let handle = tokio::spawn(async move {
            let mut conn = create_redis_connection_from_url(&url)
                .await
                .map_err(|e| format!("连接失败: {}", e))?;

            // 每个任务执行不同的操作
            let key = format!("pool_test:{}", i);
            let value = format!("value_{}", i);

            conn.set_builtin(&key, &value)
                .await
                .map_err(|e| format!("SET失败: {}", e))?;

            let retrieved = conn
                .get_builtin(&key)
                .await
                .map_err(|e| format!("GET失败: {}", e))?;

            if retrieved != Some(value.clone()) {
                return Err(format!("值不匹配: expected {}, got {:?}", value, retrieved));
            }

            info!("✅ 任务 {} 完成", i);
            Ok::<(), String>(())
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    for (i, handle) in handles.into_iter().enumerate() {
        match handle.await? {
            Ok(_) => info!("✅ 并发任务 {} 成功", i),
            Err(e) => warn!("❌ 并发任务 {} 失败: {}", i, e),
        }
    }

    info!("✅ 并发连接池测试完成");
    Ok(())
}

/// 示例3: 连接池性能测试
async fn example_pool_performance() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 示例3: 连接池性能测试");

    let redis_url = "redis://localhost:6379";
    let num_operations = 100;

    // 测试连接建立时间
    let start = Instant::now();
    let mut connection = create_redis_connection_from_url(redis_url).await?;
    let connection_time = start.elapsed();
    info!("⏱️ 连接建立时间: {:?}", connection_time);

    // 测试批量操作性能
    let start = Instant::now();
    for i in 0..num_operations {
        let key = format!("perf_test:{}", i);
        let value = format!("performance_value_{}", i);
        connection.set_builtin(&key, &value).await?;
    }
    let set_time = start.elapsed();

    let start = Instant::now();
    for i in 0..num_operations {
        let key = format!("perf_test:{}", i);
        let _ = connection.get_builtin(&key).await?;
    }
    let get_time = start.elapsed();

    info!("📊 {} 次 SET 操作总时间: {:?}", num_operations, set_time);
    info!("📊 {} 次 GET 操作总时间: {:?}", num_operations, get_time);
    info!("📊 平均 SET 时间: {:?}", set_time / num_operations);
    info!("📊 平均 GET 时间: {:?}", get_time / num_operations);

    Ok(())
}

/// 示例4: 连接池配置优化
async fn example_pool_configuration() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 示例4: 连接池配置优化");

    // 创建自定义配置
    let config = RedisConfig {
        url: "redis://localhost:6379".to_string(),
        database_index: 0,
        connection_timeout_secs: 10, // 自定义连接超时
        response_timeout_secs: 3,    // 自定义响应超时
        retry_count: 3,              // 自定义重试次数
        retry_factor_ms: 200,        // 自定义重试延迟因子
        max_retry_delay_ms: 5000,    // 自定义最大重试延迟
    };

    // 使用自定义配置创建连接
    let mut connection = RedisConnection::new(config).await?;
    connection.ping().await?;

    let stats = connection.get_pool_stats();
    info!(
        "📊 自定义配置连接池统计: 最大连接数={}, 连接超时={}秒",
        stats.max_connections, stats.connect_timeout
    );

    info!("✅ 自定义配置连接池测试完成");
    Ok(())
}

/// 示例5: 连接池健康检查
async fn example_pool_health_check() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 示例5: 连接池健康检查");

    let redis_url = "redis://localhost:6379";
    let mut connection = create_redis_connection_from_url(redis_url).await?;

    // 执行多次健康检查
    for i in 1..=5 {
        let start = Instant::now();
        connection.ping().await?;
        let ping_time = start.elapsed();

        info!("✅ 健康检查 {} - Ping 时间: {:?}", i, ping_time);

        // 稍微延迟
        sleep(tokio::time::Duration::from_millis(100)).await;
    }

    info!("✅ 连接池健康检查完成");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Redis 连接池使用示例");
    info!("📡 连接目标: redis://localhost:6379");
    info!("⚠️  请确保 Redis 服务器正在运行");

    // 运行所有示例
    let examples: Vec<(
        &str,
        Box<
            dyn Fn() -> std::pin::Pin<
                Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send>,
            >,
        >,
    )> = vec![
        (
            "基本连接池使用",
            Box::new(|| Box::pin(example_basic_pool_usage())),
        ),
        (
            "并发使用连接池",
            Box::new(|| Box::pin(example_concurrent_pool_usage())),
        ),
        (
            "连接池性能测试",
            Box::new(|| Box::pin(example_pool_performance())),
        ),
        (
            "连接池配置优化",
            Box::new(|| Box::pin(example_pool_configuration())),
        ),
        (
            "连接池健康检查",
            Box::new(|| Box::pin(example_pool_health_check())),
        ),
    ];

    let mut passed = 0;
    let total = examples.len();

    for (name, example_fn) in examples {
        info!("\n{}", "=".repeat(50));
        match example_fn().await {
            Ok(_) => {
                info!("✅ {} - 成功", name);
                passed += 1;
            }
            Err(e) => {
                warn!("❌ {} - 失败: {}", name, e);
            }
        }

        // 示例间稍微延迟
        sleep(tokio::time::Duration::from_millis(500)).await;
    }

    info!("\n{}", "=".repeat(50));
    info!("🏁 示例完成: {}/{} 成功", passed, total);

    if passed == total {
        info!("🎉 所有 Redis 连接池示例都成功运行！");
    } else {
        warn!("⚠️ 部分示例失败，请检查 Redis 服务器状态");
    }

    Ok(())
}
