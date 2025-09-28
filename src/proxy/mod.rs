//! Pingora 反向代理模块
//!
//! 提供类似 Nginx 的反向代理功能，包括：
//! - HTTP/HTTPS 代理
//! - 静态文件服务
//! - 负载均衡
//! - SSL/TLS 支持

pub mod proxy_config;
pub mod proxy_server;
pub mod proxy_service;
pub mod static_file_service;

pub use proxy_config::ProxyConfig;
pub use proxy_server::ProxyServer;
pub use proxy_service::ProxyService;
pub use static_file_service::StaticFileService;
