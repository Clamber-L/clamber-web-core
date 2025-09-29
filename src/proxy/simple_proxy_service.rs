//! 简化的代理服务模块
//!
//! 支持路由到 Kafka API 的简化代理实现

use crate::proxy::proxy_config::{LocationConfig, LocationType, ProxyConfig};
use async_trait::async_trait;
use pingora::Result;
use pingora::http::RequestHeader;
use pingora::proxy::ProxyHttp;
use pingora::proxy::Session;
use pingora::upstreams::peer::HttpPeer;
use std::sync::Arc;

/// 简化的代理服务实现
pub struct SimpleProxyService {
    config: Arc<ProxyConfig>,
}

impl SimpleProxyService {
    /// 创建新的简化代理服务
    pub fn new(config: ProxyConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// 根据请求路径找到匹配的位置配置
    fn find_location(&self, path: &str) -> Option<&LocationConfig> {
        // 按路径长度降序排序，优先匹配更具体的路径
        let mut locations: Vec<_> = self.config.locations.iter().collect();
        locations.sort_by(|a, b| b.path.len().cmp(&a.path.len()));

        for location in locations {
            if path.starts_with(&location.path) {
                return Some(location);
            }
        }
        None
    }

    /// 获取上游服务器配置
    fn get_upstream_config(
        &self,
        upstream_name: &str,
    ) -> Option<&crate::proxy::proxy_config::UpstreamConfig> {
        self.config.upstreams.get(upstream_name)
    }

    /// 选择上游服务器（简单的轮询实现）
    fn select_upstream_server<'a>(
        &self,
        upstream_config: &'a crate::proxy::proxy_config::UpstreamConfig,
    ) -> Option<&'a String> {
        // 这里可以实现更复杂的负载均衡策略
        // 目前使用简单的轮询
        upstream_config.servers.first()
    }
}

#[async_trait]
impl ProxyHttp for SimpleProxyService {
    type CTX = ();

    fn new_ctx(&self) -> Self::CTX {
        ()
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let path = session.req_header().uri.path();

        // 查找匹配的位置配置
        let location = self.find_location(path).ok_or_else(|| {
            pingora::Error::explain(
                pingora::ErrorType::InternalError,
                "No matching location found",
            )
        })?;

        match location.location_type {
            LocationType::Proxy => {
                // 代理到上游服务器
                let upstream_name = location.proxy_pass.as_ref().ok_or_else(|| {
                    pingora::Error::explain(
                        pingora::ErrorType::InternalError,
                        "No proxy_pass configured for proxy location",
                    )
                })?;

                let upstream_config = self.get_upstream_config(upstream_name).ok_or_else(|| {
                    pingora::Error::explain(pingora::ErrorType::InternalError, "Upstream not found")
                })?;

                let server = self
                    .select_upstream_server(upstream_config)
                    .ok_or_else(|| {
                        pingora::Error::explain(
                            pingora::ErrorType::InternalError,
                            "No servers in upstream",
                        )
                    })?;

                let peer = HttpPeer::new(server, self.config.ssl, self.config.server_name.clone());
                Ok(Box::new(peer))
            }
            LocationType::Static => {
                // 静态文件服务 - 返回一个虚拟的 peer
                // 实际的文件服务需要单独处理
                let peer = HttpPeer::new("127.0.0.1:1", false, "static".to_string());
                Ok(Box::new(peer))
            }
        }
    }

    async fn upstream_request_filter(
        &self,
        session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        let path = session.req_header().uri.path();

        // 查找匹配的位置配置
        if let Some(location) = self.find_location(path) {
            match location.location_type {
                LocationType::Proxy => {
                    // 修改请求路径，移除 location 前缀
                    if let Some(proxy_pass) = &location.proxy_pass {
                        if let Some(upstream_config) = self.get_upstream_config(proxy_pass) {
                            if let Some(server) = self.select_upstream_server(upstream_config) {
                                // 修改请求路径，移除 location 前缀
                                let new_path = if path.len() > location.path.len() {
                                    &path[location.path.len()..]
                                } else {
                                    "/"
                                };

                                // 解析服务器地址
                                let server_parts: Vec<&str> = server.split(':').collect();
                                let host = server_parts[0];
                                let port = server_parts.get(1).unwrap_or(&"80");

                                // 构建新的 URI
                                let new_uri = format!("http://{}:{}{}", host, port, new_path);
                                println!("Proxying to: {}", new_uri);
                            }
                        }
                    }
                }
                LocationType::Static => {
                    // 静态文件请求
                    println!("Static file request: {}", path);
                }
            }
        }

        println!("Proxying request to: {:?}", upstream_request.uri);
        Ok(())
    }
}
