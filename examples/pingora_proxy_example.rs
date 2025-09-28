//! Pingora 代理使用示例
//!
//! 展示如何使用 clamber-web-core 的 proxy 模块创建反向代理服务器

use std::collections::HashMap;
use clamber_web_core::proxy_config::{LocationConfig, LocationType, UpstreamConfig};
use clamber_web_core::{ProxyConfig, ProxyServer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建上游服务器配置
    let mut upstreams = HashMap::new();
    upstreams.insert(
        "backend".to_string(),
        UpstreamConfig {
            servers: vec!["127.0.0.1:3000".to_string()], // 后端服务地址
            lb_strategy: "roundrobin".to_string(),
        },
    );

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

    // 创建并启动代理服务器
    let mut server = ProxyServer::new(config)?;
    println!("Starting proxy server on http://{}", "0.0.0.0:8080");
    server.start()?;

    Ok(())
}
