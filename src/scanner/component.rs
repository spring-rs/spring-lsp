//! 组件扫描器模块
//!
//! 扫描项目中的所有组件定义（带有 #[derive(Service)] 的结构体）

use crate::analysis::rust::macro_analyzer::{MacroAnalyzer, SpringMacro};
use crate::protocol::types::{LocationResponse, PositionResponse, RangeResponse};
use lsp_types::Url;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// 组件扫描器
pub struct ComponentScanner {
    macro_analyzer: MacroAnalyzer,
}

impl ComponentScanner {
    /// 创建新的组件扫描器
    pub fn new() -> Self {
        Self {
            macro_analyzer: MacroAnalyzer::new(),
        }
    }

    /// 扫描项目中的所有组件
    ///
    /// # Arguments
    ///
    /// * `project_path` - 项目根目录路径
    ///
    /// # Returns
    ///
    /// 返回扫描到的所有组件信息
    pub fn scan_components(
        &self,
        project_path: &Path,
    ) -> Result<Vec<ComponentInfoResponse>, ScanError> {
        tracing::info!("Starting component scan in: {:?}", project_path);

        let mut components = Vec::new();

        // 查找 src 目录
        let src_path = project_path.join("src");
        tracing::info!("Looking for src directory: {:?}", src_path);

        if !src_path.exists() {
            tracing::error!("src directory not found at: {:?}", src_path);
            return Err(ScanError::InvalidProject(
                "src directory not found".to_string(),
            ));
        }

        tracing::info!("Found src directory, starting file scan...");
        let mut file_count = 0;

        // 遍历所有 Rust 文件
        for entry in WalkDir::new(&src_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        {
            file_count += 1;
            let file_path = entry.path();
            tracing::debug!("Scanning file: {:?}", file_path);

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

            // 提取组件信息
            for spring_macro in &rust_doc.macros {
                if let SpringMacro::DeriveService(service_macro) = spring_macro {
                    tracing::info!(
                        "Found component: {} in {:?}",
                        service_macro.struct_name,
                        file_path
                    );

                    components.push(ComponentInfoResponse {
                        name: service_macro.struct_name.clone(),
                        type_name: service_macro.struct_name.clone(),
                        scope: ComponentScope::Singleton, // spring-rs 默认是单例
                        dependencies: service_macro
                            .fields
                            .iter()
                            .filter_map(|field| {
                                // 只包含带有 inject 标注的字段
                                field.inject.as_ref().map(|_| field.type_name.clone())
                            })
                            .collect(),
                        location: LocationResponse {
                            uri: file_url.to_string(),
                            range: RangeResponse {
                                start: PositionResponse {
                                    line: service_macro.range.start.line,
                                    character: service_macro.range.start.character,
                                },
                                end: PositionResponse {
                                    line: service_macro.range.end.line,
                                    character: service_macro.range.end.character,
                                },
                            },
                        },
                    });
                }
            }
        }

        tracing::info!(
            "Scanned {} Rust files, found {} components",
            file_count,
            components.len()
        );
        Ok(components)
    }
}

impl Default for ComponentScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// 组件作用域
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentScope {
    /// 单例（默认）
    Singleton,
    /// 原型（每次注入创建新实例）
    Prototype,
}

/// 组件信息响应（用于 JSON 序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfoResponse {
    /// 组件名称
    pub name: String,
    /// 组件类型名
    #[serde(rename = "typeName")]
    pub type_name: String,
    /// 作用域
    pub scope: ComponentScope,
    /// 依赖列表
    pub dependencies: Vec<String>,
    /// 源代码位置
    pub location: LocationResponse,
}

/// spring/components 请求参数
#[derive(Debug, Deserialize)]
pub struct ComponentsRequest {
    /// 应用路径
    #[serde(rename = "appPath")]
    pub app_path: String,
}

/// spring/components 响应
#[derive(Debug, Serialize)]
pub struct ComponentsResponse {
    /// 组件列表
    pub components: Vec<ComponentInfoResponse>,
}

/// 扫描错误
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("Failed to read file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Invalid project structure: {0}")]
    InvalidProject(String),

    #[error("No components found")]
    NoComponents,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_scanner_new() {
        let scanner = ComponentScanner::new();
        // 验证扫描器创建成功
        assert!(true);
    }

    #[test]
    fn test_component_scanner_default() {
        let scanner = ComponentScanner::default();
        // 验证默认扫描器创建成功
        assert!(true);
    }
}
