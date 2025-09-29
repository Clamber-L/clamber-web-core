@echo off
REM 启动所有 Kafka 和代理服务的 Windows 脚本

echo 🚀 启动所有 Kafka 和代理服务...

REM 检查是否安装了必要的工具
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ 错误: 未找到 cargo，请先安装 Rust
    exit /b 1
)

REM 创建日志目录
if not exist logs mkdir logs

echo 📡 启动 Kafka Example API (端口 3000)...
start "Kafka API" cmd /c "cargo run --example axum_kafka_example > logs/kafka_api.log 2>&1"

echo ⚙️ 启动 Kafka Config Example API (端口 3001)...
start "Kafka Config API" cmd /c "cargo run --example axum_kafka_config_example > logs/kafka_config_api.log 2>&1"

REM 等待服务启动
echo ⏳ 等待服务启动...
timeout /t 5 /nobreak >nul

echo 🔄 启动代理服务器 (端口 8080)...
start "Proxy Server" cmd /c "cargo run --example kafka_proxy_example > logs/proxy.log 2>&1"

echo ✅ 所有服务已启动!
echo.
echo 📋 服务信息:
echo   - Kafka API: http://localhost:3000
echo   - Kafka Config API: http://localhost:3001
echo   - 代理服务器: http://localhost:8080
echo   - 静态文件: http://localhost:8080/static/
echo.
echo 🧪 测试命令:
echo   curl http://localhost:8080/api/kafka/health
echo   curl http://localhost:8080/api/config/health
echo   curl http://localhost:8080/
echo.
echo 📝 日志文件:
echo   - Kafka API: logs/kafka_api.log
echo   - Kafka Config API: logs/kafka_config_api.log
echo   - 代理服务器: logs/proxy.log
echo.
echo 💡 提示: 关闭此窗口将停止所有服务
echo.

REM 等待用户按键
pause
