//! 代理服务模块
//!
//! 实现基于 Pingora 的反向代理服务

use crate::proxy::proxy_config::ProxyConfig;
use async_trait::async_trait;
use pingora::Result;
use pingora::proxy::ProxyHttp;
use pingora::proxy::http_proxy_service;
use pingora::server::Server;
use pingora::upstreams::peer::HttpPeer;
use pingora::http::RequestHeader;
use std::sync::Arc;

/// 代理服务实现
pub struct ProxyService {
    config: Arc<ProxyConfig>,
}

impl ProxyService {
    /// 创建新的代理服务
    pub fn new(config: ProxyConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// 启动代理服务
    pub fn start(&self) -> Result<()> {
        let mut server = Server::new(None)?;
        server.bootstrap();

        // http_proxy_service expects an owned service, not a reference
        let owned_service = ProxyService::new((*self.config).clone());
        let service = http_proxy_service(&server.configuration, owned_service);
        server.add_service(service);

        // 启动服务器
        server.run(pingora::server::RunArgs::default());
        Ok(())
    }
}

#[async_trait]
impl ProxyHttp for ProxyService {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {
        ()
    }

    async fn upstream_peer(
        &self,
        _session: &mut pingora::proxy::Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        // 简单实现：选择第一个上游服务器
        // 实际实现中需要根据配置和负载均衡策略选择合适的上游服务器
        let upstream = self
            .config
            .upstreams
            .values()
            .next()
            .ok_or_else(|| pingora::Error::err::<()>(pingora::ErrorType::InternalError))
            .map_err(|_| {
                pingora::Error::explain(
                    pingora::ErrorType::InternalError,
                    "No upstream servers configured",
                )
            })?;

        let server = upstream
            .servers
            .first()
            .ok_or_else(|| pingora::Error::err::<()>(pingora::ErrorType::InternalError))
            .map_err(|_| {
                pingora::Error::explain(pingora::ErrorType::InternalError, "No servers in upstream")
            })?;

        let peer = HttpPeer::new(server, self.config.ssl, self.config.server_name.clone());
        Ok(Box::new(peer))
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut pingora::proxy::Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        // 可以在这里修改发送到上游服务器的请求
        // 例如添加头部信息、修改路径等
        println!("Proxying request to: {:?}", upstream_request.uri);
        Ok(())
    }
}
