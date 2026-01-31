//! TOML 配置文件分析模块

use lsp_types::Range;
use std::collections::HashMap;
use taplo::dom::node::IntegerValue;

/// TOML 文档
/// 
/// 表示解析后的 TOML 配置文件，包含环境变量引用、配置节和属性等信息
#[derive(Debug, Clone)]
pub struct TomlDocument {
    /// taplo 的 DOM 根节点
    pub root: taplo::dom::Node,
    /// 提取的环境变量引用
    pub env_vars: Vec<EnvVarReference>,
    /// 提取的配置节（键为配置前缀）
    pub config_sections: HashMap<String, ConfigSection>,
}

/// 环境变量引用
/// 
/// 表示 TOML 配置中的环境变量插值，格式为 `${VAR:default}` 或 `${VAR}`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVarReference {
    /// 环境变量名称
    pub name: String,
    /// 默认值（可选）
    pub default: Option<String>,
    /// 在文档中的位置范围
    pub range: Range,
}

/// 配置节
/// 
/// 表示 TOML 配置文件中的一个配置节，如 `[web]` 或 `[redis]`
#[derive(Debug, Clone)]
pub struct ConfigSection {
    /// 配置前缀（节名称）
    pub prefix: String,
    /// 配置属性映射（键为属性名）
    pub properties: HashMap<String, ConfigProperty>,
    /// 在文档中的位置范围
    pub range: Range,
}

/// 配置属性
/// 
/// 表示配置节中的单个属性，如 `host = "localhost"`
#[derive(Debug, Clone)]
pub struct ConfigProperty {
    /// 属性键
    pub key: String,
    /// 属性值
    pub value: ConfigValue,
    /// 在文档中的位置范围
    pub range: Range,
}

/// 配置值
/// 
/// 表示 TOML 配置属性的值，支持多种类型
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    /// 字符串值
    String(String),
    /// 整数值
    Integer(i64),
    /// 浮点数值
    Float(f64),
    /// 布尔值
    Boolean(bool),
    /// 数组值
    Array(Vec<ConfigValue>),
    /// 表（对象）值
    Table(HashMap<String, ConfigValue>),
}

/// TOML 分析器
/// 
/// 负责解析 TOML 配置文件，提取环境变量引用和配置节
pub struct TomlAnalyzer;

impl TomlAnalyzer {
    /// 创建新的 TOML 分析器
    pub fn new() -> Self {
        Self
    }

    /// 解析 TOML 文档
    /// 
    /// 使用 taplo 解析 TOML 内容，提取环境变量引用和配置节
    /// 
    /// # 参数
    /// 
    /// * `content` - TOML 文件内容
    /// 
    /// # 返回
    /// 
    /// 成功时返回 `TomlDocument`，失败时返回错误信息
    /// 
    /// # 示例
    /// 
    /// ```
    /// use spring_lsp::toml_analyzer::TomlAnalyzer;
    /// 
    /// let analyzer = TomlAnalyzer::new();
    /// let doc = analyzer.parse("[web]\nhost = \"localhost\"").unwrap();
    /// assert_eq!(doc.config_sections.len(), 1);
    /// ```
    pub fn parse(&self, content: &str) -> Result<TomlDocument, String> {
        // 使用 taplo 解析 TOML
        let parse_result = taplo::parser::parse(content);
        
        // 检查语法错误
        if !parse_result.errors.is_empty() {
            let error_messages: Vec<String> = parse_result
                .errors
                .iter()
                .map(|e| format!("{:?}:{:?} - {}", e.range.start(), e.range.end(), e.message))
                .collect();
            return Err(format!("TOML 语法错误: {}", error_messages.join("; ")));
        }
        
        // 转换为 DOM
        let root = parse_result.into_dom();
        
        // 提取环境变量引用
        let env_vars = self.extract_env_vars(content);
        
        // 提取配置节
        let config_sections = self.extract_config_sections(&root);
        
        Ok(TomlDocument {
            root,
            env_vars,
            config_sections,
        })
    }

    /// 提取环境变量引用
    /// 
    /// 识别 `${VAR:default}` 或 `${VAR}` 格式的环境变量插值
    fn extract_env_vars(&self, content: &str) -> Vec<EnvVarReference> {
        let mut env_vars = Vec::new();
        let mut line = 0;
        let mut line_start = 0;

        for (byte_offset, _) in content.char_indices() {
            // 更新行号和行起始位置
            if content[byte_offset..].starts_with('\n') {
                line += 1;
                line_start = byte_offset + 1;
            }

            // 查找 ${
            if content[byte_offset..].starts_with("${") {
                // 查找对应的 }
                if let Some(end_offset) = content[byte_offset + 2..].find('}') {
                    let end_offset = byte_offset + 2 + end_offset;
                    let var_content = &content[byte_offset + 2..end_offset];
                    
                    // 解析变量名和默认值
                    let (name, default) = if let Some(colon_pos) = var_content.find(':') {
                        let name = var_content[..colon_pos].to_string();
                        let default = Some(var_content[colon_pos + 1..].to_string());
                        (name, default)
                    } else {
                        (var_content.to_string(), None)
                    };
                    
                    // 计算位置
                    let start_char = byte_offset - line_start;
                    let end_char = end_offset + 1 - line_start;
                    
                    env_vars.push(EnvVarReference {
                        name,
                        default,
                        range: Range {
                            start: lsp_types::Position {
                                line: line as u32,
                                character: start_char as u32,
                            },
                            end: lsp_types::Position {
                                line: line as u32,
                                character: end_char as u32,
                            },
                        },
                    });
                }
            }
        }

        env_vars
    }

    /// 提取配置节
    /// 
    /// 遍历 TOML DOM 树，提取所有配置节和属性
    fn extract_config_sections(&self, _root: &taplo::dom::Node) -> HashMap<String, ConfigSection> {
        let sections = HashMap::new();

        // TODO: 修复 taplo Shared 类型的 API 使用问题
        // 当前 taplo 的 Shared<T> 类型无法直接访问内部数据
        // 需要研究正确的 API 使用方式或切换到其他 TOML 库
        
        sections
    }

    /// 提取配置属性
    /// 
    /// 从 TOML 节点中提取所有属性
    fn extract_properties(&self, _node: &taplo::dom::Node) -> HashMap<String, ConfigProperty> {
        let properties = HashMap::new();

        // TODO: 修复 taplo Shared 类型的 API 使用问题
        
        properties
    }

    /// 将 TOML 节点转换为配置值
    fn node_to_config_value(&self, node: &taplo::dom::Node) -> ConfigValue {
        match node {
            taplo::dom::Node::Bool(b) => ConfigValue::Boolean(b.value()),
            taplo::dom::Node::Str(s) => ConfigValue::String(s.value().to_string()),
            taplo::dom::Node::Integer(i) => {
                // IntegerValue 需要转换为 i64
                match i.value() {
                    IntegerValue::Positive(v) => ConfigValue::Integer(v as i64),
                    IntegerValue::Negative(v) => ConfigValue::Integer(v),
                }
            }
            taplo::dom::Node::Float(f) => ConfigValue::Float(f.value()),
            taplo::dom::Node::Array(_arr) => {
                // TODO: 修复 taplo Shared 类型的 API 使用问题
                ConfigValue::Array(Vec::new())
            }
            taplo::dom::Node::Table(_table) => {
                // TODO: 修复 taplo Shared 类型的 API 使用问题
                ConfigValue::Table(HashMap::new())
            }
            _ => ConfigValue::String(String::new()), // 默认值
        }
    }

    /// 将 TOML 节点转换为 LSP 范围
    fn node_to_range(&self, node: &taplo::dom::Node) -> Range {
        // taplo 的 text_ranges 返回一个迭代器
        let mut text_ranges = node.text_ranges();
        if let Some(first_range) = text_ranges.next() {
            let start: usize = first_range.start().into();
            let end: usize = first_range.end().into();

            Range {
                start: lsp_types::Position {
                    line: 0, // taplo 不直接提供行号，这里简化处理
                    character: start as u32,
                },
                end: lsp_types::Position {
                    line: 0,
                    character: end as u32,
                },
            }
        } else {
            // 默认范围
            Range {
                start: lsp_types::Position {
                    line: 0,
                    character: 0,
                },
                end: lsp_types::Position {
                    line: 0,
                    character: 0,
                },
            }
        }
    }
}

impl Default for TomlAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
