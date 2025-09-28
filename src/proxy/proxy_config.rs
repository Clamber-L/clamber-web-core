//! 代理配置模块
//!
//! 定义代理服务器的配置结构，包括监听地址、上游服务器、SSL 配置等。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 代理服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// 服务器名称
    pub server_name: String,

    /// 监听地址
    pub listen: String,

    /// 是否启用 HTTPS
    #[serde(default)]
    pub ssl: bool,

    /// SSL 证书路径
    pub ssl_cert: Option<String>,

    /// SSL 私钥路径
    pub ssl_key: Option<String>,

    /// 上游服务器配置
    pub upstreams: HashMap<String, UpstreamConfig>,

    /// 位置配置
    pub locations: Vec<LocationConfig>,
}

/// 上游服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    /// 服务器列表
    pub servers: Vec<String>,

    /// 负载均衡策略
    #[serde(default = "default_lb_strategy")]
    pub lb_strategy: String,
}

/// 位置配置（类似 Nginx 的 location 块）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationConfig {
    /// 匹配路径前缀
    pub path: String,

    /// 代理类型
    #[serde(rename = "type")]
    pub location_type: LocationType,

    /// 代理目标（用于反向代理）
    pub proxy_pass: Option<String>,

    /// 静态文件根目录（用于静态文件服务）
    pub root: Option<String>,

    /// 索引文件
    pub index: Option<Vec<String>>,
}

/// 位置类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LocationType {
    /// 反向代理
    Proxy,

    /// 静态文件服务
    Static,
}

fn default_lb_strategy() -> String {
    "roundrobin".to_string()
}
