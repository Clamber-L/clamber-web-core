//! 简化的代理服务器模块
//!
//! 支持路由到 Kafka API 的简化代理服务器

use crate::proxy::proxy_config::ProxyConfig;
use crate::proxy::simple_proxy_service::SimpleProxyService;
use pingora::Result;
use pingora::proxy::http_proxy_service;
use pingora::server::Server;
use std::sync::Arc;

/// 简化的代理服务器
pub struct SimpleProxyServer {
    config: Arc<ProxyConfig>,
    server: Server,
}

impl SimpleProxyServer {
    /// 创建新的简化代理服务器
    pub fn new(config: ProxyConfig) -> Result<Self> {
        let server = Server::new(None)?;
        Ok(Self {
            config: Arc::new(config),
            server,
        })
    }

    /// 启动简化代理服务器
    pub fn start(&mut self) -> Result<()> {
        self.server.bootstrap();

        // 创建简化代理服务
        let proxy_service = SimpleProxyService::new((*self.config).clone());
        let service = http_proxy_service(&self.server.configuration, proxy_service);

        // 添加服务到服务器
        self.server.add_service(service);

        println!("Simple proxy server starting on {}", self.config.listen);
        println!("Server name: {}", self.config.server_name);
        println!("SSL enabled: {}", self.config.ssl);

        // 打印位置配置
        for location in &self.config.locations {
            match location.location_type {
                crate::proxy::proxy_config::LocationType::Proxy => {
                    println!(
                        "Proxy location: {} -> {}",
                        location.path,
                        location.proxy_pass.as_ref().unwrap_or(&"none".to_string())
                    );
                }
                crate::proxy::proxy_config::LocationType::Static => {
                    println!(
                        "Static location: {} -> {}",
                        location.path,
                        location.root.as_ref().unwrap_or(&"none".to_string())
                    );
                }
            }
        }

        // 启动服务器
        let server = std::mem::replace(&mut self.server, Server::new(None)?);
        server.run(pingora::server::RunArgs::default());
        Ok(())
    }

    /// 停止简化代理服务器
    pub fn stop(&mut self) {
        println!("Stopping simple proxy server...");
    }

    /// 获取配置
    pub fn get_config(&self) -> &ProxyConfig {
        &self.config
    }
}
