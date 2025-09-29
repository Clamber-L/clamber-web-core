#!/bin/bash

# 测试代理服务器功能的脚本

echo "🧪 测试 Kafka API 代理服务器功能..."
echo ""

# 等待服务启动
echo "⏳ 等待服务启动..."
sleep 3

# 测试函数
test_endpoint() {
    local url=$1
    local description=$2
    local method=${3:-GET}
    local data=$4
    
    echo "🔍 测试: $description"
    echo "   URL: $url"
    
    if [ "$method" = "POST" ] && [ -n "$data" ]; then
        response=$(curl -s -X POST -H "Content-Type: application/json" -d "$data" "$url")
    else
        response=$(curl -s "$url")
    fi
    
    if [ $? -eq 0 ]; then
        echo "   ✅ 成功"
        echo "   响应: $response"
    else
        echo "   ❌ 失败"
    fi
    echo ""
}

# 测试静态文件服务
test_endpoint "http://localhost:8080/" "静态文件 - 根路径"

# 测试 Kafka API 代理
test_endpoint "http://localhost:8080/api/kafka/health" "Kafka API - 健康检查"
test_endpoint "http://localhost:8080/api/kafka/producer-stats" "Kafka API - 生产者统计"
test_endpoint "http://localhost:8080/api/kafka/consumer-stats" "Kafka API - 消费者统计"

# 测试发送消息
test_endpoint "http://localhost:8080/api/kafka/send-message" "Kafka API - 发送消息" "POST" '{"topic": "test-topic", "key": "test-key", "message": "Hello from proxy test!"}'

# 测试配置 API 代理
test_endpoint "http://localhost:8080/api/config/health" "Config API - 健康检查"
test_endpoint "http://localhost:8080/api/config/producer-stats" "Config API - 生产者统计"

# 测试发送消息到配置 API
test_endpoint "http://localhost:8080/api/config/send-message" "Config API - 发送消息" "POST" '{"topic": "config-topic", "key": "config-key", "message": "Hello from config proxy test!"}'

echo "🎉 测试完成!"
echo ""
echo "💡 提示:"
echo "  - 如果测试失败，请确保所有服务都已启动"
echo "  - 检查日志文件: logs/*.log"
echo "  - 确保 Kafka 服务器正在运行"
