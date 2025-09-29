//! Kafka API 代理服务器示例
//!
//! 使用 Pingora 实现代理服务器，将请求转发到 Kafka example API 和静态文件服务

use clamber_web_core::proxy::{ProxyConfig, SimpleProxyServer};
use clamber_web_core::proxy_config::{LocationConfig, LocationType, UpstreamConfig};
use std::collections::HashMap;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 启动 Kafka API 代理服务器...");

    // 从配置文件加载配置
    let config = load_config_from_file("examples/kafka_proxy_config.yaml")?;

    // 创建并启动简化代理服务器
    let mut server = SimpleProxyServer::new(config)?;

    println!("✅ 代理服务器配置加载成功");
    println!("📡 开始启动代理服务器...");

    server.start()?;

    Ok(())
}

/// 从 YAML 文件加载配置
fn load_config_from_file(path: &str) -> Result<ProxyConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: ProxyConfig = serde_yaml::from_str(&content)?;
    Ok(config)
}

/// 创建默认配置（备用方案）
#[allow(dead_code)]
fn create_default_config() -> ProxyConfig {
    // 创建上游服务器配置
    let mut upstreams = HashMap::new();

    // Kafka API 上游服务器
    upstreams.insert(
        "kafka_api".to_string(),
        UpstreamConfig {
            servers: vec!["127.0.0.1:3000".to_string()],
            lb_strategy: "roundrobin".to_string(),
        },
    );

    // Kafka Config API 上游服务器
    upstreams.insert(
        "kafka_config_api".to_string(),
        UpstreamConfig {
            servers: vec!["127.0.0.1:3001".to_string()],
            lb_strategy: "roundrobin".to_string(),
        },
    );

    // 创建位置配置
    let locations = vec![
        // API 路由 - 转发到 Kafka example
        LocationConfig {
            path: "/api/kafka/".to_string(),
            location_type: LocationType::Proxy,
            proxy_pass: Some("kafka_api".to_string()),
            root: None,
            index: None,
        },
        // 配置 API 路由 - 转发到 Kafka config example
        LocationConfig {
            path: "/api/config/".to_string(),
            location_type: LocationType::Proxy,
            proxy_pass: Some("kafka_config_api".to_string()),
            root: None,
            index: None,
        },
        // 静态文件服务
        LocationConfig {
            path: "/static/".to_string(),
            location_type: LocationType::Static,
            proxy_pass: None,
            root: Some("./static".to_string()),
            index: Some(vec!["index.html".to_string(), "index.htm".to_string()]),
        },
        // 根路径 - 提供默认页面
        LocationConfig {
            path: "/".to_string(),
            location_type: LocationType::Static,
            proxy_pass: None,
            root: Some("./static".to_string()),
            index: Some(vec!["index.html".to_string(), "index.htm".to_string()]),
        },
    ];

    // 创建代理配置
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

/// 测试配置加载
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        // 测试默认配置创建
        let config = create_default_config();
        assert_eq!(config.server_name, "kafka-proxy.local");
        assert_eq!(config.listen, "0.0.0.0:8080");
        assert_eq!(config.locations.len(), 4);
    }

    #[test]
    fn test_config_file_loading() {
        // 测试配置文件加载
        let result = load_config_from_file("examples/kafka_proxy_config.yaml");
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.server_name, "kafka-proxy.local");
        assert_eq!(config.listen, "0.0.0.0:8080");
    }
}
