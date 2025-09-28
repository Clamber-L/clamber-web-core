# Pingora 代理指南

本指南介绍如何在 clamber-web-core 中使用 Pingora 代理功能，该功能提供了类似 Nginx 的反向代理和静态文件服务功能。

## 功能特性

- HTTP/HTTPS 反向代理
- 负载均衡
- 静态文件服务
- SSL/TLS 支持
- 类似 Nginx 的配置方式

## 启用 Pingora 功能

要使用 Pingora 代理功能，需要在 `Cargo.toml` 中启用 `pingora` 特性：

```toml
[dependencies]
clamber-web-core = { version = "0.1.2", features = ["pingora"] }
```

或者启用所有功能：

```toml
[dependencies]
clamber-web-core = { version = "0.1.2", features = ["full"] }
```

## 配置代理

### 1. 创建代理配置

```rust
use clamber_web_core::pingora::proxy_config::{ProxyConfig, UpstreamConfig, LocationConfig, LocationType};
use std::collections::HashMap;

// 创建上游服务器配置
let mut upstreams = HashMap::new();
upstreams.insert("backend".to_string(), UpstreamConfig {
    servers: vec!["127.0.0.1:3000".to_string()], // 后端服务地址
    lb_strategy: "roundrobin".to_string(),
});

// 创建位置配置
let locations = vec![
    LocationConfig {
        path: "/api/".to_string(),
        location_type: LocationType::Proxy,
        proxy_pass: Some("backend".to_string()), // 代理到 backend 上游
        root: None,
        index: None,
    },
    LocationConfig {
        path: "/static/".to_string(),
        location_type: LocationType::Static,
        proxy_pass: None,
        root: Some("./static".to_string()), // 静态文件根目录
        index: Some(vec!["index.html".to_string()]),
    },
];

// 创建代理配置
let config = ProxyConfig {
    server_name: "example.com".to_string(),
    listen: "0.0.0.0:8080".to_string(), // 监听地址
    ssl: false,
    ssl_cert: None,
    ssl_key: None,
    upstreams,
    locations,
};
```

### 2. 启动代理服务器

```rust
use clamber_web_core::pingora::proxy_server::ProxyServer;

// 创建并启动代理服务器
let mut server = ProxyServer::new(config)?;
server.start()?;
```

## 运行示例

项目中包含了一个完整的示例，可以通过以下命令运行：

```bash
cargo run --example pingora_proxy_example --features proxy
```

## 配置说明

### ProxyConfig

| 字段 | 类型 | 描述 |
|------|------|------|
| server_name | String | 服务器名称 |
| listen | String | 监听地址，如 "0.0.0.0:8080" |
| ssl | bool | 是否启用 HTTPS |
| ssl_cert | Option<String> | SSL 证书路径 |
| ssl_key | Option<String> | SSL 私钥路径 |
| upstreams | HashMap<String, UpstreamConfig> | 上游服务器配置 |
| locations | Vec<LocationConfig> | 位置配置 |

### UpstreamConfig

| 字段 | 类型 | 描述 |
|------|------|------|
| servers | Vec<String> | 服务器列表 |
| lb_strategy | String | 负载均衡策略（默认为 "roundrobin"） |

### LocationConfig

| 字段 | 类型 | 描述 |
|------|------|------|
| path | String | 匹配路径前缀 |
| location_type | LocationType | 位置类型（Proxy 或 Static） |
| proxy_pass | Option<String> | 代理目标（用于反向代理） |
| root | Option<String> | 静态文件根目录（用于静态文件服务） |
| index | Option<Vec<String>> | 索引文件列表 |

## 高级功能

### SSL/TLS 配置

要启用 HTTPS，需要设置 SSL 相关配置：

```rust
let config = ProxyConfig {
    server_name: "example.com".to_string(),
    listen: "0.0.0.0:8443".to_string(),
    ssl: true,
    ssl_cert: Some("/path/to/cert.pem".to_string()),
    ssl_key: Some("/path/to/key.pem".to_string()),
    // ... 其他配置
};
```

### 负载均衡

目前支持轮询（roundrobin）负载均衡策略。可以在 [UpstreamConfig](file:///e:/project/rust/clamber-web-core/src/pingora/proxy_config.rs#L23-L32) 中配置：

```rust
UpstreamConfig {
    servers: vec![
        "127.0.0.1:3000".to_string(),
        "127.0.0.1:3001".to_string(),
        "127.0.0.1:3002".to_string(),
    ],
    lb_strategy: "roundrobin".to_string(),
}
```

## 注意事项

1. 确保防火墙允许配置的端口通信
2. 对于生产环境，建议使用有效的 SSL 证书
3. 静态文件服务会自动防止路径遍历攻击
4. 代理功能支持 HTTP/1.1 和 HTTP/2