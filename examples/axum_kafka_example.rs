//! Axum + Kafka 集成示例
//!
//! 演示如何在 axum 项目中使用 clamber-web-core 的 Kafka 功能

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use clamber_web_core::kafka::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::task;

#[derive(Debug, Serialize, Deserialize)]
struct UserMessage {
    user_id: u64,
    message: String,
    timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageRequest {
    topic: String,
    key: Option<String>,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserMessageRequest {
    topic: String,
    user_id: u64,
    message: String,
}

/// 应用状态
type AppState = Arc<KafkaAppState>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("启动 Axum + Kafka 示例应用...");

    // 创建 Kafka AppState
    let kafka_state = create_default_kafka_app_state(
        vec!["localhost:9092".to_string()],
        "axum-example-group".to_string(),
    )
    .await?;

    println!("Kafka AppState 创建成功");

    // 启动轮询消费者服务
    start_polling_consumer(Arc::new(kafka_state.clone())).await;

    // 创建 axum 路由
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/send-message", post(send_message))
        .route("/send-user-message", post(send_user_message))
        .route("/producer-stats", get(get_producer_stats))
        .route("/consumer-stats", get(get_consumer_stats))
        .with_state(Arc::new(kafka_state));

    // 启动服务器
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("服务器启动在 http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

/// 根路径处理器
async fn root() -> &'static str {
    "Axum + Kafka 示例应用运行中！\n\n可用端点:\n- GET /health - 健康检查\n- POST /send-message - 发送消息\n- POST /send-user-message - 发送用户消息\n- GET /producer-stats - 获取生产者统计\n- GET /consumer-stats - 获取消费者统计"
}

/// 健康检查处理器
async fn health_check(State(_state): State<AppState>) -> Json<ApiResponse> {
    Json(ApiResponse {
        success: true,
        message: "服务运行正常".to_string(),
    })
}

/// 发送消息处理器
async fn send_message(
    State(state): State<AppState>,
    Json(payload): Json<MessageRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    match state
        .send_message(&payload.topic, payload.key.as_deref(), &payload.message)
        .await
    {
        Ok(_) => Ok(Json(ApiResponse {
            success: true,
            message: "消息发送成功".to_string(),
        })),
        Err(e) => {
            eprintln!("发送消息失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 发送用户消息处理器
async fn send_user_message(
    State(state): State<AppState>,
    Json(payload): Json<UserMessageRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    let user_message = UserMessage {
        user_id: payload.user_id,
        message: payload.message,
        timestamp: chrono::Utc::now().timestamp(),
    };

    let key = format!("user_{}", payload.user_id);

    match state
        .send_serialized(&payload.topic, Some(&key), &user_message)
        .await
    {
        Ok(_) => Ok(Json(ApiResponse {
            success: true,
            message: "用户消息发送成功".to_string(),
        })),
        Err(e) => {
            eprintln!("发送用户消息失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取生产者统计信息处理器
async fn get_producer_stats(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse>, StatusCode> {
    match state.get_producer_stats() {
        Ok(stats) => Ok(Json(ApiResponse {
            success: true,
            message: stats,
        })),
        Err(e) => {
            eprintln!("获取生产者统计失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取消费者统计信息处理器
async fn get_consumer_stats(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse>, StatusCode> {
    match state.get_consumer_stats().await {
        Ok(stats) => Ok(Json(ApiResponse {
            success: true,
            message: stats,
        })),
        Err(e) => {
            eprintln!("获取消费者统计失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 启动轮询消费者服务
async fn start_polling_consumer(state: AppState) {
    let topics = vec![
        "test-topic".to_string(),
        "user-messages".to_string(),
        "notifications".to_string(),
    ];

    let polling_service = PollingConsumerService::new(
        (*state).clone(),
        topics,
        Duration::from_secs(1), // 每秒轮询一次
        10,                     // 每次最多处理10条消息
    );

    // 在后台任务中启动轮询
    task::spawn(async move {
        let message_handler = |message: OwnedMessage| -> KafkaResult<()> {
            let topic = message.topic();
            let partition = message.partition();
            let offset = message.offset();
            let key = message
                .key()
                .map(|k| String::from_utf8_lossy(k).to_string());
            let payload = message
                .payload()
                .map(|p| String::from_utf8_lossy(p).to_string());

            println!(
                "收到消息 - Topic: {}, Partition: {}, Offset: {}, Key: {:?}, Payload: {:?}",
                topic, partition, offset, key, payload
            );

            // 根据主题处理不同类型的消息
            match topic {
                "user-messages" => {
                    if let Some(payload_str) = payload {
                        if let Ok(user_msg) = serde_json::from_str::<UserMessage>(&payload_str) {
                            println!("处理用户消息: {:?}", user_msg);
                            // 这里可以添加具体的业务逻辑
                        }
                    }
                }
                "notifications" => {
                    println!("处理通知消息: {:?}", payload);
                    // 这里可以添加通知处理逻辑
                }
                _ => {
                    println!("处理通用消息: {:?}", payload);
                }
            }

            Ok(())
        };

        if let Err(e) = polling_service
            .start_polling_with_timeout(message_handler, Duration::from_secs(5))
            .await
        {
            eprintln!("轮询消费者服务错误: {}", e);
        }
    });

    println!("轮询消费者服务已启动");
}

/// 测试函数：发送一些示例消息
#[allow(dead_code)]
async fn send_test_messages(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    println!("发送测试消息...");

    // 发送简单文本消息
    state
        .send_message("test-topic", Some("test_key"), "Hello from Axum!")
        .await?;

    // 发送用户消息
    let user_message = UserMessage {
        user_id: 12345,
        message: "Hello from user!".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };

    state
        .send_serialized("user-messages", Some("user_12345"), &user_message)
        .await?;

    // 发送通知消息
    state
        .send_message("notifications", Some("notification_1"), "系统维护通知")
        .await?;

    println!("测试消息发送完成");
    Ok(())
}

/// 测试函数：轮询一些消息
#[allow(dead_code)]
async fn test_polling(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    println!("测试轮询消息...");

    // 订阅主题
    state.subscribe(&["test-topic", "user-messages"]).await?;

    // 轮询几条消息
    for i in 0..5 {
        match state.poll_message(Duration::from_secs(3)).await? {
            Some(message) => {
                println!(
                    "轮询到消息 {}: topic={}, payload={:?}",
                    i + 1,
                    message.topic(),
                    String::from_utf8_lossy(message.payload().unwrap_or(&[]))
                );
            }
            None => {
                println!("轮询消息 {}: 超时", i + 1);
            }
        }
    }

    Ok(())
}
