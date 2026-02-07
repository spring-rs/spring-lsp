//! 配置扫描器模块
//!
//! 扫描项目中所有带有 `#[derive(Configurable)]` 的配置结构体

use crate::Result;
use lsp_types::{Location, Position, Range, Url};
use serde::{Deserialize, Serialize};
use std::path::Path;
use syn::spanned::Spanned;
use walkdir::WalkDir;

/// 配置扫描请求
#[derive(Debug, Deserialize)]
pub struct ConfigurationsRequest {
    /// 应用路径
    #[serde(rename = "appPath")]
    pub app_path: String,
}

/// 配置扫描响应
#[derive(Debug, Serialize)]
pub struct ConfigurationsResponse {
    /// 配置结构列表
    pub configurations: Vec<ConfigurationStruct>,
}

/// 配置结构信息
#[derive(Debug, Clone, Serialize)]
pub struct ConfigurationStruct {
    /// 结构体名称
    pub name: String,
    /// 配置前缀（从 #[config_prefix = "..."] 提取）
    pub prefix: String,
    /// 字段列表
    pub fields: Vec<ConfigField>,
    /// 定义位置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
}

/// 配置字段信息
#[derive(Debug, Clone, Serialize)]
pub struct ConfigField {
    /// 字段名称
    pub name: String,
    /// 字段类型
    #[serde(rename = "type")]
    pub type_name: String,
    /// 是否可选
    pub optional: bool,
    /// 描述（从文档注释提取）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 配置扫描器
pub struct ConfigScanner;

impl ConfigScanner {
    /// 创建新的配置扫描器
    pub fn new() -> Self {
        Self
    }

    /// 扫描项目中的所有配置结构
    ///
    /// # Arguments
    ///
    /// * `project_path` - 项目根目录路径
    ///
    /// # Returns
    ///
    /// 返回找到的所有配置结构列表
    pub fn scan_configurations(&self, project_path: &Path) -> Result<Vec<ConfigurationStruct>> {
        tracing::info!("Scanning configurations in: {:?}", project_path);

        let mut configurations = Vec::new();

        // 遍历项目中的所有 Rust 文件
        for entry in WalkDir::new(project_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // 跳过 target 目录
            if path.components().any(|c| c.as_os_str() == "target") {
                continue;
            }

            // 只处理 .rs 文件
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            // 读取文件内容
            let content = match std::fs::read_to_string(path) {
                Ok(content) => content,
                Err(e) => {
                    tracing::warn!("Failed to read file {:?}: {}", path, e);
                    continue;
                }
            };

            // 解析文件
            let syntax_tree = match syn::parse_file(&content) {
                Ok(tree) => tree,
                Err(e) => {
                    tracing::debug!("Failed to parse file {:?}: {}", path, e);
                    continue;
                }
            };

            // 提取配置结构
            if let Ok(file_configs) = self.extract_configurations_from_file(&syntax_tree, path) {
                configurations.extend(file_configs);
            }
        }

        tracing::info!("Found {} configuration structs", configurations.len());
        Ok(configurations)
    }

    /// 从单个文件中提取配置结构
    fn extract_configurations_from_file(
        &self,
        syntax_tree: &syn::File,
        file_path: &Path,
    ) -> Result<Vec<ConfigurationStruct>> {
        let mut configurations = Vec::new();

        // 遍历所有项
        for item in &syntax_tree.items {
            if let syn::Item::Struct(item_struct) = item {
                // 检查是否有 Configurable derive
                if self.has_configurable_derive(item_struct) {
                    if let Some(config) = self.extract_configuration_struct(item_struct, file_path)
                    {
                        configurations.push(config);
                    }
                }
            }
        }

        Ok(configurations)
    }

    /// 检查结构体是否派生了 Configurable
    fn has_configurable_derive(&self, item_struct: &syn::ItemStruct) -> bool {
        for attr in &item_struct.attrs {
            if attr.path().is_ident("derive") {
                if let Ok(meta_list) = attr.meta.require_list() {
                    let tokens_str = meta_list.tokens.to_string();
                    if tokens_str.contains("Configurable") {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// 提取配置结构信息
    fn extract_configuration_struct(
        &self,
        item_struct: &syn::ItemStruct,
        file_path: &Path,
    ) -> Option<ConfigurationStruct> {
        // 提取配置前缀
        let prefix = self.extract_config_prefix(item_struct)?;

        // 提取字段
        let fields = self.extract_fields(&item_struct.fields);

        // 提取文档注释
        let _doc_comment = self.extract_doc_comment(&item_struct.attrs);

        // 构建位置信息
        let location = self.build_location(item_struct, file_path);

        Some(ConfigurationStruct {
            name: item_struct.ident.to_string(),
            prefix,
            fields,
            location,
        })
    }

    /// 提取配置前缀
    fn extract_config_prefix(&self, item_struct: &syn::ItemStruct) -> Option<String> {
        for attr in &item_struct.attrs {
            if attr.path().is_ident("config_prefix") {
                // 解析 #[config_prefix = "prefix"]
                if let Ok(meta_name_value) = attr.meta.require_name_value() {
                    if let syn::Expr::Lit(expr_lit) = &meta_name_value.value {
                        if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                            return Some(lit_str.value());
                        }
                    }
                }
            }
        }
        None
    }

    /// 提取结构体字段
    fn extract_fields(&self, fields: &syn::Fields) -> Vec<ConfigField> {
        let mut result = Vec::new();

        if let syn::Fields::Named(fields_named) = fields {
            for field in &fields_named.named {
                if let Some(ident) = &field.ident {
                    // 提取字段类型
                    let type_name = self.type_to_string(&field.ty);

                    // 检查是否是 Option<T>
                    let optional = self.is_option_type(&field.ty);

                    // 提取文档注释
                    let description = self.extract_doc_comment(&field.attrs);

                    result.push(ConfigField {
                        name: ident.to_string(),
                        type_name,
                        optional,
                        description,
                    });
                }
            }
        }

        result
    }

    /// 将类型转换为字符串
    fn type_to_string(&self, ty: &syn::Type) -> String {
        match ty {
            syn::Type::Path(type_path) => {
                // 提取类型路径的最后一段
                if let Some(segment) = type_path.path.segments.last() {
                    let ident = segment.ident.to_string();

                    // 处理泛型参数
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        let generic_args: Vec<String> = args
                            .args
                            .iter()
                            .filter_map(|arg| {
                                if let syn::GenericArgument::Type(ty) = arg {
                                    Some(self.type_to_string(ty))
                                } else {
                                    None
                                }
                            })
                            .collect();

                        if !generic_args.is_empty() {
                            return format!("{}<{}>", ident, generic_args.join(", "));
                        }
                    }

                    ident
                } else {
                    "Unknown".to_string()
                }
            }
            syn::Type::Reference(type_ref) => {
                format!("&{}", self.type_to_string(&type_ref.elem))
            }
            syn::Type::Tuple(type_tuple) => {
                let elem_types: Vec<String> = type_tuple
                    .elems
                    .iter()
                    .map(|ty| self.type_to_string(ty))
                    .collect();
                format!("({})", elem_types.join(", "))
            }
            _ => "Unknown".to_string(),
        }
    }

    /// 检查类型是否是 Option<T>
    fn is_option_type(&self, ty: &syn::Type) -> bool {
        if let syn::Type::Path(type_path) = ty {
            if let Some(segment) = type_path.path.segments.last() {
                return segment.ident == "Option";
            }
        }
        false
    }

    /// 提取文档注释
    fn extract_doc_comment(&self, attrs: &[syn::Attribute]) -> Option<String> {
        let mut doc_lines = Vec::new();

        for attr in attrs {
            if attr.path().is_ident("doc") {
                if let Ok(meta_name_value) = attr.meta.require_name_value() {
                    if let syn::Expr::Lit(expr_lit) = &meta_name_value.value {
                        if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                            let line = lit_str.value().trim().to_string();
                            if !line.is_empty() {
                                doc_lines.push(line);
                            }
                        }
                    }
                }
            }
        }

        if doc_lines.is_empty() {
            None
        } else {
            Some(doc_lines.join(" "))
        }
    }

    /// 构建位置信息
    fn build_location(&self, item_struct: &syn::ItemStruct, file_path: &Path) -> Option<Location> {
        // 将文件路径转换为 URI
        let uri = match Url::from_file_path(file_path) {
            Ok(uri) => uri,
            Err(_) => {
                tracing::warn!("Failed to convert path to URI: {:?}", file_path);
                return None;
            }
        };

        // 获取结构体的 span
        let span = item_struct.span();
        let start = span.start();
        let end = span.end();

        // 构建 Range
        let range = Range {
            start: Position {
                line: start.line.saturating_sub(1) as u32, // syn 的行号从 1 开始
                character: start.column as u32,
            },
            end: Position {
                line: end.line.saturating_sub(1) as u32,
                character: end.column as u32,
            },
        };

        Some(Location { uri, range })
    }
}

impl Default for ConfigScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_config_prefix() {
        let code = r#"
            #[derive(Debug, Configurable, Deserialize)]
            #[config_prefix = "database"]
            struct DatabaseConfig {
                host: String,
                port: u16,
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let scanner = ConfigScanner::new();

        if let syn::Item::Struct(item_struct) = &syntax_tree.items[0] {
            let prefix = scanner.extract_config_prefix(item_struct);
            assert_eq!(prefix, Some("database".to_string()));
        } else {
            panic!("Expected struct item");
        }
    }

    #[test]
    fn test_extract_fields() {
        let code = r#"
            struct Config {
                /// Database host
                host: String,
                /// Database port
                port: u16,
                /// Optional timeout
                timeout: Option<u64>,
            }
        "#;

        let syntax_tree = syn::parse_file(code).unwrap();
        let scanner = ConfigScanner::new();

        if let syn::Item::Struct(item_struct) = &syntax_tree.items[0] {
            let fields = scanner.extract_fields(&item_struct.fields);

            assert_eq!(fields.len(), 3);

            assert_eq!(fields[0].name, "host");
            assert_eq!(fields[0].type_name, "String");
            assert!(!fields[0].optional);
            assert_eq!(fields[0].description, Some("Database host".to_string()));

            assert_eq!(fields[1].name, "port");
            assert_eq!(fields[1].type_name, "u16");
            assert!(!fields[1].optional);

            assert_eq!(fields[2].name, "timeout");
            assert_eq!(fields[2].type_name, "Option<u64>");
            assert!(fields[2].optional);
        } else {
            panic!("Expected struct item");
        }
    }

    #[test]
    fn test_is_option_type() {
        let scanner = ConfigScanner::new();

        // Option<String>
        let ty: syn::Type = syn::parse_str("Option<String>").unwrap();
        assert!(scanner.is_option_type(&ty));

        // String
        let ty: syn::Type = syn::parse_str("String").unwrap();
        assert!(!scanner.is_option_type(&ty));

        // Vec<String>
        let ty: syn::Type = syn::parse_str("Vec<String>").unwrap();
        assert!(!scanner.is_option_type(&ty));
    }
}
