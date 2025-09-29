# Kafka API 代理服务器集成指南

本指南介绍如何使用 Pingora 代理服务器将请求转发到 Kafka example API 和静态文件服务。

## 概述

我们创建了一个基于 Pingora 的代理服务器，可以：

- 🔄 **反向代理**: 将 API 请求转发到 Kafka example 服务
- 📁 **静态文件服务**: 提供静态文件访问
- ⚙️ **配置管理**: 支持 YAML 配置文件
- 🚀 **高性能**: 基于 Cloudflare 的 Pingora 框架

## 架构图

```
客户端请求
    ↓
代理服务器 (端口 8080)
    ↓
┌─────────────────┬─────────────────┐
│   API 路由      │   静态文件      │
│                 │                 │
│ /api/kafka/*    │ /static/*       │
│ /api/config/*   │ /              │
│                 │                 │
└─────────────────┴─────────────────┘
    ↓                    ↓
Kafka API          静态文件目录
(端口 3000)        (./static/)
    ↓
Kafka Config API
(端口 3001)
```

## 快速开始

### 1. 启动所有服务

#### Linux/macOS
```bash
# 给脚本执行权限
chmod +x examples/start_all_services.sh

# 启动所有服务
./examples/start_all_services.sh
```

#### Windows
```cmd
# 启动所有服务
examples\start_all_services.bat
```

### 2. 测试代理功能

#### Linux/macOS
```bash
# 给脚本执行权限
chmod +x examples/test_proxy.sh

# 运行测试
./examples/test_proxy.sh
```

#### Windows
```cmd
# 运行测试
examples\test_proxy.bat
```

### 3. 手动测试

```bash
# 测试静态文件
curl http://localhost:8080/

# 测试 Kafka API 健康检查
curl http://localhost:8080/api/kafka/health

# 测试发送消息
curl -X POST http://localhost:8080/api/kafka/send-message \
  -H "Content-Type: application/json" \
  -d '{"topic": "test-topic", "key": "test-key", "message": "Hello from proxy!"}'

# 测试配置 API
curl http://localhost:8080/api/config/health
```

## 配置说明

### 代理配置文件 (`examples/kafka_proxy_config.yaml`)

```yaml
# 服务器配置
server_name: "kafka-proxy.local"
listen: "0.0.0.0:8080"
ssl: false

# 上游服务器配置
upstreams:
  kafka_api:
    servers:
      - "127.0.0.1:3000"  # Kafka example API
    lb_strategy: "roundrobin"
  
  kafka_config_api:
    servers:
      - "127.0.0.1:3001"  # Kafka config example API
    lb_strategy: "roundrobin"

# 位置配置
locations:
  # API 路由
  - path: "/api/kafka/"
    type: "proxy"
    proxy_pass: "kafka_api"
  
  - path: "/api/config/"
    type: "proxy"
    proxy_pass: "kafka_config_api"
  
  # 静态文件服务
  - path: "/static/"
    type: "static"
    root: "./static"
    index: ["index.html", "index.htm"]
  
  # 根路径
  - path: "/"
    type: "static"
    root: "./static"
    index: ["index.html", "index.htm"]
```

## API 路由

### Kafka API (转发到端口 3000)

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | `/api/kafka/` | Kafka API 根路径 |
| GET | `/api/kafka/health` | 健康检查 |
| POST | `/api/kafka/send-message` | 发送消息 |
| POST | `/api/kafka/send-user-message` | 发送用户消息 |
| GET | `/api/kafka/producer-stats` | 生产者统计 |
| GET | `/api/kafka/consumer-stats` | 消费者统计 |

### 配置 API (转发到端口 3001)

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | `/api/config/` | 配置 API 根路径 |
| GET | `/api/config/health` | 健康检查 |
| POST | `/api/config/send-message` | 发送消息 |
| GET | `/api/config/producer-stats` | 生产者统计 |
| GET | `/api/config/consumer-stats` | 消费者统计 |

### 静态文件

| 路径 | 描述 |
|------|------|
| `/` | 主页面 (index.html) |
| `/static/` | 静态文件目录 |
| `/static/README.md` | 静态文件说明 |

## 代码示例

### 基本使用

```rust
use clamber_web_core::proxy::{SimpleProxyServer, ProxyConfig};
use clamber_web_core::proxy_config::{UpstreamConfig, LocationConfig, LocationType};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建配置
    let config = create_proxy_config();
    
    // 创建并启动代理服务器
    let mut server = SimpleProxyServer::new(config)?;
    server.start()?;
    
    Ok(())
}

fn create_proxy_config() -> ProxyConfig {
    let mut upstreams = HashMap::new();
    
    // 添加 Kafka API 上游
    upstreams.insert(
        "kafka_api".to_string(),
        UpstreamConfig {
            servers: vec!["127.0.0.1:3000".to_string()],
            lb_strategy: "roundrobin".to_string(),
        },
    );
    
    // 创建位置配置
    let locations = vec![
        LocationConfig {
            path: "/api/kafka/".to_string(),
            location_type: LocationType::Proxy,
            proxy_pass: Some("kafka_api".to_string()),
            root: None,
            index: None,
        },
    ];
    
    ProxyConfig {
        server_name: "kafka-proxy.local".to_string(),
        listen: "0.0.0.0:8080".to_string(),
        ssl: false,
        ssl_cert: None,
        ssl_key: None,
        upstreams,
        locations,
    }
}
```

### 从配置文件加载

```rust
use std::fs;

fn load_config_from_file(path: &str) -> Result<ProxyConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: ProxyConfig = serde_yaml::from_str(&content)?;
    Ok(config)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config_from_file("examples/kafka_proxy_config.yaml")?;
    let mut server = SimpleProxyServer::new(config)?;
    server.start()?;
    Ok(())
}
```

## 服务管理

### 启动服务

1. **Kafka Example API** (端口 3000)
   ```bash
   cargo run --example axum_kafka_example
   ```

2. **Kafka Config Example API** (端口 3001)
   ```bash
   cargo run --example axum_kafka_config_example
   ```

3. **代理服务器** (端口 8080)
   ```bash
   cargo run --example kafka_proxy_example
   ```

### 日志文件

- `logs/kafka_api.log` - Kafka API 日志
- `logs/kafka_config_api.log` - Kafka Config API 日志
- `logs/proxy.log` - 代理服务器日志

### 进程管理

启动脚本会创建 PID 文件：
- `logs/kafka_api.pid`
- `logs/kafka_config.pid`
- `logs/proxy.pid`

## 故障排除

### 常见问题

1. **端口冲突**
   - 确保端口 3000、3001、8080 未被占用
   - 使用 `netstat -tulpn | grep :8080` 检查端口状态

2. **Kafka 连接失败**
   - 确保 Kafka 服务器正在运行
   - 检查 Kafka 服务器地址和端口

3. **静态文件无法访问**
   - 确保 `./static` 目录存在
   - 检查文件权限

4. **代理转发失败**
   - 检查上游服务器是否正在运行
   - 验证配置文件中的服务器地址

### 调试技巧

1. **启用详细日志**
   ```bash
   RUST_LOG=debug cargo run --example kafka_proxy_example
   ```

2. **检查服务状态**
   ```bash
   # 检查进程
   ps aux | grep kafka
   
   # 检查端口
   netstat -tulpn | grep -E ":(3000|3001|8080)"
   ```

3. **测试网络连接**
   ```bash
   # 测试 Kafka API
   curl -v http://localhost:3000/health
   
   # 测试代理
   curl -v http://localhost:8080/api/kafka/health
   ```

## 性能优化

### 代理服务器优化

1. **连接池配置**
   ```yaml
   upstreams:
     kafka_api:
       servers: ["127.0.0.1:3000"]
       lb_strategy: "roundrobin"
       # 可以添加更多配置参数
   ```

2. **负载均衡**
   - 支持多个上游服务器
   - 可配置负载均衡策略

3. **缓存配置**
   - 静态文件缓存
   - 响应头优化

### Kafka 优化

1. **批量处理**
   - 调整批量大小
   - 优化轮询间隔

2. **连接管理**
   - 连接池配置
   - 超时设置

## 扩展功能

### 添加新的 API 路由

1. 在配置文件中添加新的上游服务器
2. 添加对应的位置配置
3. 重启代理服务器

### SSL/TLS 支持

```yaml
ssl: true
ssl_cert: "/path/to/cert.pem"
ssl_key: "/path/to/key.pem"
```

### 自定义中间件

可以扩展 `SimpleProxyService` 来添加：
- 认证中间件
- 日志中间件
- 限流中间件
- 监控中间件

## 总结

通过这个代理服务器，你可以：

✅ **统一入口**: 通过一个端口访问所有服务  
✅ **路由管理**: 灵活的路由配置  
✅ **静态文件**: 内置静态文件服务  
✅ **高性能**: 基于 Pingora 的高性能代理  
✅ **易于配置**: YAML 配置文件  
✅ **易于扩展**: 模块化设计  

这个解决方案为你的 Kafka API 提供了完整的代理和静态文件服务功能！
