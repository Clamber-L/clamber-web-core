//! 静态文件服务模块
//!
//! 提供静态文件服务功能，类似 Nginx 的静态文件服务

use std::io::Result;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// 静态文件服务
pub struct StaticFileService {
    root: PathBuf,
}

impl StaticFileService {
    /// 创建新的静态文件服务
    pub fn new(root: &str) -> Self {
        Self {
            root: PathBuf::from(root),
        }
    }

    /// 处理静态文件请求
    pub async fn serve_file(&self, path: &str) -> Result<Vec<u8>> {
        // 防止路径遍历攻击
        let full_path = self.sanitize_path(path)?;

        // 检查文件是否存在
        if !full_path.exists() || !full_path.is_file() {
            // 简化处理：直接返回文本内容，避免依赖未解析的 http 类型
            return Ok(b"Not Found".to_vec());
        }

        // 读取文件内容
        let mut file = File::open(&full_path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        // 简化处理：返回文件字节内容
        Ok(buffer)
    }

    /// 清理路径，防止路径遍历攻击
    fn sanitize_path(&self, path: &str) -> Result<PathBuf> {
        // 移除查询参数和片段
        let clean_path = path.split('?').next().unwrap_or(path);
        let clean_path = clean_path.split('#').next().unwrap_or(clean_path);

        // 解析路径
        let path = Path::new(clean_path);

        // 规范化路径
        let mut full_path = self.root.clone();
        for component in path.components() {
            match component {
                std::path::Component::Normal(part) => {
                    full_path.push(part);
                }
                _ => {
                    // 忽略其他组件（如 .. 或 .）
                    continue;
                }
            }
        }

        Ok(full_path)
    }

    /// 根据文件扩展名猜测内容类型
    fn guess_content_type(&self, path: &Path) -> &'static str {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("html") | Some("htm") => "text/html",
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("json") => "application/json",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("ico") => "image/x-icon",
            Some("txt") => "text/plain",
            Some("xml") => "application/xml",
            Some("pdf") => "application/pdf",
            _ => "application/octet-stream",
        }
    }
}
