//! Redis 连接测试用例
//!
//! 测试 clamber-web-core Redis 模块的各种功能
//! 包括功能性测试、性能测试、并发测试、错误处理测试四个主要维度

use clamber_web_core::redis::{
    RedisConfig, RedisConnection, create_redis_connection_from_config,
    create_redis_connection_from_url,
};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{error, info, warn};

// Redis 连接配置 - 使用本地Redis实例进行测试
const REDIS_HOST: &str = "127.0.0.1";
const REDIS_PORT: u16 = 6379;
const REDIS_DATABASE: u8 = 0;
const REDIS_PASSWORD: &str = "";

/// 构建 Redis 连接 URL
fn build_redis_url_with_auth() -> String {
    format!(
        "redis://:{}@{}:{}/{}",
        REDIS_PASSWORD, REDIS_HOST, REDIS_PORT, REDIS_DATABASE
    )
}

/// 测试 1: 基本连接测试
async fn test_basic_connection() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试 1: 基本 Redis 连接");

    let redis_url = build_redis_url_with_auth();
    let mut connection = create_redis_connection_from_url(&redis_url).await?;

    // 测试连接
    connection.ping().await?;
    info!("✅ 基本连接测试成功");

    Ok(())
}

/// 测试 2: RedisConnection 结构体测试
async fn test_redis_connection_struct() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试 2: RedisConnection 结构体功能");

    let config = RedisConfig {
        url: build_redis_url_with_auth(),
        database_index: 0,
        connection_timeout_secs: 5,
        response_timeout_secs: 5,
        retry_count: 5,
        retry_factor_ms: 5,
        max_retry_delay_ms: 5,
    };

    let mut redis_conn = RedisConnection::new(config.clone()).await?;

    // 测试 ping
    redis_conn.ping().await?;
    info!("✅ RedisConnection ping 测试成功");

    // 测试便利函数
    let mut conn2 = create_redis_connection_from_config(config).await?;
    conn2.ping().await?;
    info!("✅ create_redis_connection_from_config 测试成功");

    Ok(())
}

/// 测试 3: Redis 配置测试
async fn test_redis_config() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试 3: Redis 配置功能");

    // 测试从 URL 创建配置
    let url_config = RedisConfig::from_url(build_redis_url_with_auth());
    assert_eq!(url_config.url, build_redis_url_with_auth());
    info!("✅ 从 URL 创建配置测试成功");

    // 测试配置验证
    let mut invalid_config = RedisConfig::default();
    invalid_config.url = String::new();
    assert!(invalid_config.validate().is_err());
    info!("✅ 配置验证测试成功");

    // 测试 URL 构建
    let mut config = RedisConfig::from_url("redis://localhost:6379");
    assert_eq!(config.build_url(), "redis://localhost:6379");

    config.database_index = 1;
    assert_eq!(config.build_url(), "redis://localhost:6379/1");
    info!("✅ URL 构建测试成功");

    Ok(())
}

/// 测试 4: 便利函数测试
async fn test_convenience_functions() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试 4: 便利函数");

    let redis_url = build_redis_url_with_auth();

    // 测试直接从 URL 创建连接
    let mut conn1 = create_redis_connection_from_url(&redis_url).await?;
    conn1.ping().await?;
    info!("✅ create_redis_connection_from_url 测试成功");

    // 测试从配置创建连接
    let config = RedisConfig::from_url(&redis_url);
    let mut conn2 = create_redis_connection_from_config(config).await?;
    conn2.ping().await?;
    info!("✅ create_redis_connection_from_config 测试成功");

    Ok(())
}

/// 测试 5: Redis 基本操作测试
async fn test_basic_operations() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试 5: Redis 基本操作");

    let redis_url = build_redis_url_with_auth();
    let mut connection = create_redis_connection_from_url(&redis_url).await?;

    // 使用时间戳生成唯一键名，防止冲突
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_key = format!("test:basic:key:{}", timestamp);
    let test_value = "test_value_123";

    // 测试 SET 操作
    connection.set_builtin(&test_key, test_value).await?;
    info!("✅ SET 操作测试成功");

    // 测试 GET 操作
    let retrieved_value = connection.get_builtin(&test_key).await?;
    assert_eq!(retrieved_value, Some(test_value.to_string()));
    info!("✅ GET 操作测试成功: {}", retrieved_value.unwrap());

    // 测试 EXISTS 操作
    let exists = connection.exists_builtin(&test_key).await?;
    assert!(exists);
    info!("✅ EXISTS 操作测试成功: 键存在");

    // 测试不存在的键
    let non_existent_key = format!("test:basic:nonexistent:{}", timestamp);
    let non_existent_value = connection.get_builtin(&non_existent_key).await?;
    assert_eq!(non_existent_value, None);
    info!("✅ 获取不存在键测试成功");

    let not_exists = connection.exists_builtin(&non_existent_key).await?;
    assert!(!not_exists);
    info!("✅ 检查不存在键测试成功");

    // 测试覆盖写入
    let new_value = "updated_value_456";
    connection.set_builtin(&test_key, new_value).await?;
    let updated_value = connection.get_builtin(&test_key).await?;
    assert_eq!(updated_value, Some(new_value.to_string()));
    info!("✅ 覆盖写入测试成功: {}", updated_value.unwrap());

    Ok(())
}

/// 测试 6: Redis 列表操作测试
async fn test_list_operations() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试 6: Redis 列表操作");

    let redis_url = build_redis_url_with_auth();
    let mut connection = create_redis_connection_from_url(&redis_url).await?;

    // 使用时间戳生成唯一列表键名
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let list_key = format!("test:list:items:{}", timestamp);
    let item1 = "item_1";
    let item2 = "item_2";
    let item3 = "item_3";

    // 测试 LPUSH 操作
    let length1 = connection.lpush(&list_key, item1).await?;
    assert_eq!(length1, 1);
    info!("✅ LPUSH 第一个元素测试成功, 列表长度: {}", length1);

    let length2 = connection.lpush(&list_key, item2).await?;
    assert_eq!(length2, 2);
    info!("✅ LPUSH 第二个元素测试成功, 列表长度: {}", length2);

    let length3 = connection.lpush(&list_key, item3).await?;
    assert_eq!(length3, 3);
    info!("✅ LPUSH 第三个元素测试成功, 列表长度: {}", length3);

    // 测试 RPOP 操作（列表是 LIFO，所以应该先弹出 item1）
    let popped1 = connection.rpop(&list_key).await?;
    assert_eq!(popped1, Some(item1.to_string()));
    info!("✅ RPOP 第一次测试成功: {}", popped1.unwrap());

    let popped2 = connection.rpop(&list_key).await?;
    assert_eq!(popped2, Some(item2.to_string()));
    info!("✅ RPOP 第二次测试成功: {}", popped2.unwrap());

    let popped3 = connection.rpop(&list_key).await?;
    assert_eq!(popped3, Some(item3.to_string()));
    info!("✅ RPOP 第三次测试成功: {}", popped3.unwrap());

    // 测试空列表弹出
    let empty_pop = connection.rpop(&list_key).await?;
    assert_eq!(empty_pop, None);
    info!("✅ 空列表 RPOP 测试成功");

    Ok(())
}

/// 测试 7: Redis 哈希操作测试
async fn test_hash_operations() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试 7: Redis 哈希操作");

    let redis_url = build_redis_url_with_auth();
    let mut connection = create_redis_connection_from_url(&redis_url).await?;

    // 使用唯一的哈希键名，防止与之前的测试冲突
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let hash_key = format!("test:hash:user:{}", timestamp);
    let field1 = "name";
    let value1 = "John Doe";
    let field2 = "age";
    let value2 = "30";
    let field3 = "email";
    let value3 = "john@example.com";

    // 测试 HSET 操作
    let is_new1 = connection.hset(&hash_key, field1, value1).await?;
    // 第一次设置新字段应该返回 true，但根据 Redis 文档，返回值表示是否为新字段
    info!(
        "✅ HSET {}={} 测试成功, is_new: {}",
        field1, value1, is_new1
    );

    let is_new2 = connection.hset(&hash_key, field2, value2).await?;
    info!(
        "✅ HSET {}={} 测试成功, is_new: {}",
        field2, value2, is_new2
    );

    let is_new3 = connection.hset(&hash_key, field3, value3).await?;
    info!(
        "✅ HSET {}={} 测试成功, is_new: {}",
        field3, value3, is_new3
    );

    // 测试 HGET 操作
    let retrieved_value1 = connection.hget(&hash_key, field1).await?;
    assert_eq!(retrieved_value1, Some(value1.to_string()));
    info!("✅ HGET {} 测试成功: {}", field1, retrieved_value1.unwrap());

    let retrieved_value2 = connection.hget(&hash_key, field2).await?;
    assert_eq!(retrieved_value2, Some(value2.to_string()));
    info!("✅ HGET {} 测试成功: {}", field2, retrieved_value2.unwrap());

    let retrieved_value3 = connection.hget(&hash_key, field3).await?;
    assert_eq!(retrieved_value3, Some(value3.to_string()));
    info!("✅ HGET {} 测试成功: {}", field3, retrieved_value3.unwrap());

    // 测试更新存在的字段
    let new_age = "31";
    let is_new_update = connection.hset(&hash_key, field2, new_age).await?;
    // 更新存在的字段应该返回 false
    info!("✅ HSET 更新存在字段测试成功, is_new: {}", is_new_update);

    let updated_age = connection.hget(&hash_key, field2).await?;
    assert_eq!(updated_age, Some(new_age.to_string()));
    info!("✅ HGET 获取更新后的值测试成功: {}", updated_age.unwrap());

    // 测试获取不存在的字段
    let non_existent_field = "non_existent";
    let non_existent_value = connection.hget(&hash_key, non_existent_field).await?;
    assert_eq!(non_existent_value, None);
    info!("✅ HGET 不存在字段测试成功");

    Ok(())
}

/// 测试 8: 错误处理测试
async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试 8: 错误处理");

    // 测试无效的连接 URL
    let invalid_url = "redis://invalid:invalid@invalid:3306/invalid";

    match create_redis_connection_from_url(invalid_url).await {
        Ok(_) => {
            error!("❌ 预期连接失败，但连接成功了");
            return Err("连接到无效地址应该失败".into());
        }
        Err(e) => {
            info!("✅ 正确处理了无效连接: {}", e);
        }
    }

    // 测试无效配置
    let invalid_config = RedisConfig {
        url: String::new(), // 空 URL
        ..RedisConfig::default()
    };

    match RedisConnection::new(invalid_config).await {
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

/// 测试 10: 连接性能测试
async fn test_connection_performance() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试 10: 连接性能测试");

    let redis_url = build_redis_url_with_auth();

    // 测试连接建立时间
    let start = Instant::now();
    let mut connection = create_redis_connection_from_url(&redis_url).await?;
    let connection_time = start.elapsed();

    // 测试 ping 响应时间
    let start = Instant::now();
    connection.ping().await?;
    let ping_time = start.elapsed();

    info!("⏱️ 连接建立时间: {:?}", connection_time);
    info!("⏱️ Ping 响应时间: {:?}", ping_time);

    if ping_time < Duration::from_millis(100) {
        info!("✅ 连接性能良好 (< 100ms)");
    } else {
        warn!("⚠️ 连接响应较慢 (> 100ms)");
    }

    // 测试基本操作性能
    let test_key = "test:perf:key";
    let test_value = "test_performance_value";
    let num_operations = 100;

    let start = Instant::now();
    for i in 0..num_operations {
        let key = format!("{}:{}", test_key, i);
        connection
            .set_builtin(&key, &format!("{}{}", test_value, i))
            .await?;
    }
    let set_time = start.elapsed();

    let start = Instant::now();
    for i in 0..num_operations {
        let key = format!("{}:{}", test_key, i);
        let _ = connection.get_builtin(&key).await?;
    }
    let get_time = start.elapsed();

    info!("📊 {} 次 SET 操作总时间: {:?}", num_operations, set_time);
    info!("📊 {} 次 GET 操作总时间: {:?}", num_operations, get_time);
    info!("📊 平均 SET 时间: {:?}", set_time / num_operations);
    info!("📊 平均 GET 时间: {:?}", get_time / num_operations);

    Ok(())
}

/// 测试 11: 并发连接测试
async fn test_concurrent_connections() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试 11: 并发连接测试");

    let redis_url = build_redis_url_with_auth();
    let mut handles = vec![];
    let num_connections = 5;

    // 创建多个并发连接
    for i in 0..num_connections {
        let url = redis_url.clone();
        let handle = tokio::spawn(async move {
            let mut connection = create_redis_connection_from_url(&url)
                .await
                .map_err(|e| format!("连接失败: {}", e))?;

            // 测试连接
            connection
                .ping()
                .await
                .map_err(|e| format!("ping失败: {}", e))?;

            // 执行一些操作
            let test_key = format!("test:concurrent:{}:key", i);
            let test_value = format!("concurrent_value_{}", i);

            connection
                .set_builtin(&test_key, &test_value)
                .await
                .map_err(|e| format!("set失败: {}", e))?;

            let retrieved = connection
                .get_builtin(&test_key)
                .await
                .map_err(|e| format!("get失败: {}", e))?;

            if retrieved != Some(test_value.clone()) {
                return Err(format!(
                    "值不匹配: expected {}, got {:?}",
                    test_value, retrieved
                ));
            }

            info!("✅ 并发连接 {} 成功", i + 1);
            Ok::<(), String>(())
        });
        handles.push(handle);
    }

    // 等待所有连接完成
    for (i, handle) in handles.into_iter().enumerate() {
        let _result = handle.await.map_err(|e| format!("任务执行失败: {}", e))??;
        info!("✅ 并发任务 {} 完成", i + 1);
    }

    info!("✅ 并发连接测试完成");
    Ok(())
}

/// 测试 12: 并发操作测试
async fn test_concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试 12: 并发操作测试");

    let redis_url = build_redis_url_with_auth();
    let mut connection = create_redis_connection_from_url(&redis_url).await?;

    // 预先设置一些数据
    let num_keys = 10;
    for i in 0..num_keys {
        let key = format!("test:concurrent_ops:key:{}", i);
        let value = format!("initial_value_{}", i);
        connection.set_builtin(&key, &value).await?;
    }

    let mut handles = vec![];
    let num_workers = 3;

    // 创建多个并发操作任务
    for worker_id in 0..num_workers {
        let url = redis_url.clone();
        let handle = tokio::spawn(async move {
            let mut conn = create_redis_connection_from_url(&url)
                .await
                .map_err(|e| format!("Worker {} 连接失败: {}", worker_id, e))?;

            // 每个 worker 操作不同的键
            for i in 0..num_keys {
                if i % num_workers == worker_id {
                    let key = format!("test:concurrent_ops:key:{}", i);

                    // 读取当前值
                    let _current = conn
                        .get_builtin(&key)
                        .await
                        .map_err(|e| format!("Worker {} 读取失败: {}", worker_id, e))?;

                    // 更新值
                    let new_value = format!("updated_by_worker_{}_{}", worker_id, i);
                    conn.set_builtin(&key, &new_value)
                        .await
                        .map_err(|e| format!("Worker {} 更新失败: {}", worker_id, e))?;

                    // 验证更新
                    let updated = conn
                        .get_builtin(&key)
                        .await
                        .map_err(|e| format!("Worker {} 验证失败: {}", worker_id, e))?;

                    if updated != Some(new_value.clone()) {
                        return Err(format!(
                            "Worker {} 值不匹配: expected {}, got {:?}",
                            worker_id, new_value, updated
                        ));
                    }
                }
            }

            info!("✅ Worker {} 完成所有操作", worker_id);
            Ok::<(), String>(())
        });
        handles.push(handle);
    }

    // 等待所有 worker 完成
    for (i, handle) in handles.into_iter().enumerate() {
        let _result = handle
            .await
            .map_err(|e| format!("Worker {} 任务失败: {}", i, e))??;
    }

    info!("✅ 并发操作测试完成");
    Ok(())
}

/// 测试 13: Redis 健康检查测试
async fn test_health_check() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试 13: Redis 健康检查");

    let redis_url = build_redis_url_with_auth();
    let mut connection = create_redis_connection_from_url(&redis_url).await?;

    // 测试多次 ping 来检查连接稳定性
    for i in 1..=5 {
        let start = Instant::now();
        connection.ping().await?;
        let ping_time = start.elapsed();
        info!("✅ 健康检查 {} - Ping 时间: {:?}", i, ping_time);

        // 稍微延迟再次检查
        sleep(Duration::from_millis(100)).await;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 开始 Redis 连接测试");
    info!("📡 连接目标: {}:{}", REDIS_HOST, REDIS_PORT);
    info!("📄 数据库索引: {}", REDIS_DATABASE);
    info!("🗺️ 测试覆盖维度: 功能性、性能、并发、错误处理");

    println!("");
    info!(
        "⚠️  注意: 请确保 Redis 服务器在 {}:{} 上运行",
        REDIS_HOST, REDIS_PORT
    );
    info!("⚠️  如果需要密码，请修改 build_redis_url_with_auth_with_auth 函数");
    println!("");

    // 运行所有测试
    type TestFn = fn() -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send>,
    >;

    let tests: Vec<(&str, TestFn)> = vec![
        ("基本连接测试", || Box::pin(test_basic_connection())),
        ("RedisConnection 结构体测试", || {
            Box::pin(test_redis_connection_struct())
        }),
        ("Redis 配置测试", || Box::pin(test_redis_config())),
        ("便利函数测试", || {
            Box::pin(test_convenience_functions())
        }),
        ("Redis 基本操作测试", || {
            Box::pin(test_basic_operations())
        }),
        ("Redis 列表操作测试", || {
            Box::pin(test_list_operations())
        }),
        ("Redis 哈希操作测试", || {
            Box::pin(test_hash_operations())
        }),
        ("错误处理测试", || Box::pin(test_error_handling())),
        ("连接性能测试", || {
            Box::pin(test_connection_performance())
        }),
        ("并发连接测试", || {
            Box::pin(test_concurrent_connections())
        }),
        ("并发操作测试", || {
            Box::pin(test_concurrent_operations())
        }),
        ("Redis 健康检查测试", || Box::pin(test_health_check())),
    ];

    let mut passed = 0;
    let mut failed = 0;
    let mut failed_tests = Vec::new();

    for (test_name, test_fn) in tests {
        println!("");
        info!("🏁 正在执行: {}", test_name);

        match test_fn().await {
            Ok(_) => {
                info!("✅ {} - 成功", test_name);
                passed += 1;
            }
            Err(e) => {
                error!("❌ {} - 失败: {}", test_name, e);
                failed += 1;
                failed_tests.push((test_name, e.to_string()));
            }
        }

        // 测试间稍微延迟
        sleep(Duration::from_millis(200)).await;
    }

    println!("");
    info!("🎆 Redis 测试结果总结");
    info!("✅ 成功: {} 个测试", passed);
    info!("❌ 失败: {} 个测试", failed);

    if !failed_tests.is_empty() {
        println!("");
        error!("🚨 失败的测试详情:");
        for (test_name, error_msg) in failed_tests {
            error!("  - {}: {}", test_name, error_msg);
        }
    }

    if failed == 0 {
        info!("🎉 所有 Redis 测试都已成功通过！");
        Ok(())
    } else {
        Err(format!("{} 个测试失败", failed).into())
    }
}
