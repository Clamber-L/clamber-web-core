//! 代理服务器模块
//!
//! 负责启动和管理代理服务器实例

use crate::proxy::proxy_config::ProxyConfig;
use crate::proxy::proxy_service::ProxyService;
use pingora::Result;
use pingora::proxy::http_proxy_service;
use pingora::server::Server;
use std::sync::Arc;

/// 代理服务器
pub struct ProxyServer {
    config: Arc<ProxyConfig>,
    server: Server,
}

impl ProxyServer {
    /// 创建新的代理服务器
    pub fn new(config: ProxyConfig) -> Result<Self> {
        let server = Server::new(None)?;
        Ok(Self {
            config: Arc::new(config),
            server,
        })
    }

    /// 启动代理服务器
    pub fn start(&mut self) -> Result<()> {
        self.server.bootstrap();

        // 创建代理服务
        let proxy_service = ProxyService::new((*self.config).clone());
        let service = http_proxy_service(&self.server.configuration, proxy_service);

        // 添加服务到服务器
        self.server.add_service(service);

        // 启动服务器
        let server = std::mem::replace(&mut self.server, Server::new(None)?);
        server.run(pingora::server::RunArgs::default());
        Ok(())
    }

    /// 停止代理服务器
    pub fn stop(&mut self) {
        // 在实际实现中，这里需要添加优雅关闭的逻辑
        println!("Stopping proxy server...");
    }
}
