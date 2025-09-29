#!/bin/bash

# 启动所有 Kafka 和代理服务的脚本

echo "🚀 启动所有 Kafka 和代理服务..."

# 检查是否安装了必要的工具
if ! command -v cargo &> /dev/null; then
    echo "❌ 错误: 未找到 cargo，请先安装 Rust"
    exit 1
fi

# 创建日志目录
mkdir -p logs

echo "📡 启动 Kafka Example API (端口 3000)..."
cargo run --example axum_kafka_example > logs/kafka_api.log 2>&1 &
KAFKA_API_PID=$!

echo "⚙️ 启动 Kafka Config Example API (端口 3001)..."
cargo run --example axum_kafka_config_example > logs/kafka_config_api.log 2>&1 &
KAFKA_CONFIG_PID=$!

# 等待服务启动
echo "⏳ 等待服务启动..."
sleep 5

echo "🔄 启动代理服务器 (端口 8080)..."
cargo run --example kafka_proxy_example > logs/proxy.log 2>&1 &
PROXY_PID=$!

echo "✅ 所有服务已启动!"
echo ""
echo "📋 服务信息:"
echo "  - Kafka API: http://localhost:3000"
echo "  - Kafka Config API: http://localhost:3001"
echo "  - 代理服务器: http://localhost:8080"
echo "  - 静态文件: http://localhost:8080/static/"
echo ""
echo "🧪 测试命令:"
echo "  curl http://localhost:8080/api/kafka/health"
echo "  curl http://localhost:8080/api/config/health"
echo "  curl http://localhost:8080/"
echo ""
echo "📝 日志文件:"
echo "  - Kafka API: logs/kafka_api.log"
echo "  - Kafka Config API: logs/kafka_config_api.log"
echo "  - 代理服务器: logs/proxy.log"
echo ""
echo "🛑 停止所有服务:"
echo "  kill $KAFKA_API_PID $KAFKA_CONFIG_PID $PROXY_PID"
echo ""

# 保存 PID 到文件
echo $KAFKA_API_PID > logs/kafka_api.pid
echo $KAFKA_CONFIG_PID > logs/kafka_config.pid
echo $PROXY_PID > logs/proxy.pid

echo "💡 提示: 按 Ctrl+C 停止所有服务"

# 等待用户中断
trap 'echo "🛑 停止所有服务..."; kill $KAFKA_API_PID $KAFKA_CONFIG_PID $PROXY_PID 2>/dev/null; exit 0' INT

# 保持脚本运行
wait
