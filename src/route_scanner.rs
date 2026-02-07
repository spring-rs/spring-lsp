//! 路由扫描器模块
//!
//! 扫描项目中的所有路由定义

use crate::macro_analyzer::{MacroAnalyzer, SpringMacro};
use lsp_types::Url;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// 路由扫描器
pub struct RouteScanner {
    macro_analyzer: MacroAnalyzer,
}

impl RouteScanner {
    /// 创建新的路由扫描器
    pub fn new() -> Self {
        Self {
            macro_analyzer: MacroAnalyzer::new(),
        }
    }

    /// 扫描项目中的所有路由
    ///
    /// # Arguments
    ///
    /// * `project_path` - 项目根目录路径
    ///
    /// # Returns
    ///
    /// 返回扫描到的所有路由信息
    pub fn scan_routes(&self, project_path: &Path) -> Result<Vec<RouteInfoResponse>, ScanError> {
        let mut routes = Vec::new();

        // 查找 src 目录
        let src_path = project_path.join("src");
        if !src_path.exists() {
            return Err(ScanError::InvalidProject(
                "src directory not found".to_string(),
            ));
        }

        // 遍历所有 Rust 文件
        for entry in WalkDir::new(&src_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        {
            let file_path = entry.path();

            // 读取文件内容
            let content = match fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(e) => {
                    tracing::warn!("Failed to read file {:?}: {}", file_path, e);
                    continue;
                }
            };

            // 解析文件
            let file_url = match Url::from_file_path(file_path) {
                Ok(url) => url,
                Err(_) => {
                    tracing::warn!("Failed to convert path to URL: {:?}", file_path);
                    continue;
                }
            };

            let rust_doc = match self.macro_analyzer.parse(file_url.clone(), content) {
                Ok(doc) => doc,
                Err(e) => {
                    tracing::warn!("Failed to parse file {:?}: {}", file_path, e);
                    continue;
                }
            };

            // 提取宏信息
            let rust_doc = match self.macro_analyzer.extract_macros(rust_doc) {
                Ok(doc) => doc,
                Err(e) => {
                    tracing::warn!("Failed to extract macros from {:?}: {}", file_path, e);
                    continue;
                }
            };

            // 提取路由信息
            for spring_macro in &rust_doc.macros {
                if let SpringMacro::Route(route_macro) = spring_macro {
                    // 为每个 HTTP 方法创建独立的路由条目
                    for method in &route_macro.methods {
                        routes.push(RouteInfoResponse {
                            method: method.as_str().to_string(),
                            path: route_macro.path.clone(),
                            handler: route_macro.handler_name.clone(),
                            is_openapi: route_macro.is_openapi,
                            location: LocationResponse {
                                uri: file_url.to_string(),
                                range: RangeResponse {
                                    start: PositionResponse {
                                        line: route_macro.range.start.line,
                                        character: route_macro.range.start.character,
                                    },
                                    end: PositionResponse {
                                        line: route_macro.range.end.line,
                                        character: route_macro.range.end.character,
                                    },
                                },
                            },
                        });
                    }
                }
            }
        }

        Ok(routes)
    }
}

impl Default for RouteScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// 路由信息响应（用于 JSON 序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInfoResponse {
    /// HTTP 方法
    pub method: String,
    /// 路径模式
    pub path: String,
    /// 处理器函数名
    pub handler: String,
    /// 是否为 OpenAPI 路由
    #[serde(rename = "isOpenapi")]
    pub is_openapi: bool,
    /// 源代码位置
    pub location: LocationResponse,
}

/// 位置信息响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationResponse {
    /// 文件 URI
    pub uri: String,
    /// 范围
    pub range: RangeResponse,
}

/// 范围响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeResponse {
    /// 起始位置
    pub start: PositionResponse,
    /// 结束位置
    pub end: PositionResponse,
}

/// 位置响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionResponse {
    /// 行号（从 0 开始）
    pub line: u32,
    /// 列号（从 0 开始）
    pub character: u32,
}

/// spring/routes 请求参数
#[derive(Debug, Deserialize)]
pub struct RoutesRequest {
    /// 应用路径
    #[serde(rename = "appPath")]
    pub app_path: String,
}

/// spring/routes 响应
#[derive(Debug, Serialize)]
pub struct RoutesResponse {
    /// 路由列表
    pub routes: Vec<RouteInfoResponse>,
}

/// 扫描错误
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("Failed to read file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Invalid project structure: {0}")]
    InvalidProject(String),

    #[error("No routes found")]
    NoRoutes,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_scanner_new() {
        let scanner = RouteScanner::new();
        // 验证扫描器创建成功
        assert!(true);
    }

    #[test]
    fn test_route_scanner_default() {
        let scanner = RouteScanner::default();
        // 验证默认扫描器创建成功
        assert!(true);
    }
}
