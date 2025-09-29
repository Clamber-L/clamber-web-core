//! Axum + Kafka 配置文件示例
//!
//! 演示如何使用配置文件创建 Kafka AppState

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

/// 应用状态
type AppState = Arc<KafkaAppState>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("启动 Axum + Kafka 配置文件示例应用...");

    // 从配置文件创建 Kafka AppState
    let kafka_state = create_kafka_app_state_from_config(
        "examples/axum_kafka_producer_config.yaml",
        "examples/axum_kafka_consumer_config.yaml",
    )
    .await?;

    println!("Kafka AppState 从配置文件创建成功");

    // 启动轮询消费者服务
    start_polling_consumer(Arc::new(kafka_state.clone())).await;

    // 创建 axum 路由
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/send-message", post(send_message))
        .route("/producer-stats", get(get_producer_stats))
        .route("/consumer-stats", get(get_consumer_stats))
        .with_state(Arc::new(kafka_state));

    // 启动服务器
    let listener = TcpListener::bind("0.0.0.0:3001").await?;
    println!("服务器启动在 http://0.0.0.0:3001");

    axum::serve(listener, app).await?;

    Ok(())
}

/// 根路径处理器
async fn root() -> &'static str {
    "Axum + Kafka 配置文件示例应用运行中！\n\n可用端点:\n- GET /health - 健康检查\n- POST /send-message - 发送消息\n- GET /producer-stats - 获取生产者统计\n- GET /consumer-stats - 获取消费者统计"
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
    let topics = vec!["test-topic".to_string(), "config-example".to_string()];

    let polling_service = PollingConsumerService::new(
        (*state).clone(),
        topics,
        Duration::from_secs(2), // 每2秒轮询一次
        5,                      // 每次最多处理5条消息
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
                "[配置文件示例] 收到消息 - Topic: {}, Partition: {}, Offset: {}, Key: {:?}, Payload: {:?}",
                topic, partition, offset, key, payload
            );

            Ok(())
        };

        if let Err(e) = polling_service
            .start_polling_with_timeout(message_handler, Duration::from_secs(10))
            .await
        {
            eprintln!("轮询消费者服务错误: {}", e);
        }
    });

    println!("轮询消费者服务已启动");
}
