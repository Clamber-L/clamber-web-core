@echo off
REM 测试代理服务器功能的 Windows 脚本

echo 🧪 测试 Kafka API 代理服务器功能...
echo.

REM 等待服务启动
echo ⏳ 等待服务启动...
timeout /t 3 /nobreak >nul

REM 测试函数
:test_endpoint
set url=%1
set description=%2
set method=%3
set data=%4

echo 🔍 测试: %description%
echo    URL: %url%

if "%method%"=="POST" (
    if defined data (
        curl -s -X POST -H "Content-Type: application/json" -d "%data%" "%url%"
    ) else (
        curl -s -X POST "%url%"
    )
) else (
    curl -s "%url%"
)

if %errorlevel% equ 0 (
    echo    ✅ 成功
) else (
    echo    ❌ 失败
)
echo.
goto :eof

REM 测试静态文件服务
call :test_endpoint "http://localhost:8080/" "静态文件 - 根路径"

REM 测试 Kafka API 代理
call :test_endpoint "http://localhost:8080/api/kafka/health" "Kafka API - 健康检查"
call :test_endpoint "http://localhost:8080/api/kafka/producer-stats" "Kafka API - 生产者统计"
call :test_endpoint "http://localhost:8080/api/kafka/consumer-stats" "Kafka API - 消费者统计"

REM 测试发送消息
call :test_endpoint "http://localhost:8080/api/kafka/send-message" "Kafka API - 发送消息" "POST" "{\"topic\": \"test-topic\", \"key\": \"test-key\", \"message\": \"Hello from proxy test!\"}"

REM 测试配置 API 代理
call :test_endpoint "http://localhost:8080/api/config/health" "Config API - 健康检查"
call :test_endpoint "http://localhost:8080/api/config/producer-stats" "Config API - 生产者统计"

REM 测试发送消息到配置 API
call :test_endpoint "http://localhost:8080/api/config/send-message" "Config API - 发送消息" "POST" "{\"topic\": \"config-topic\", \"key\": \"config-key\", \"message\": \"Hello from config proxy test!\"}"

echo 🎉 测试完成!
echo.
echo 💡 提示:
echo   - 如果测试失败，请确保所有服务都已启动
echo   - 检查日志文件: logs\*.log
echo   - 确保 Kafka 服务器正在运行

pause
