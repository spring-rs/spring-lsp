//! 配置 Schema 管理模块

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

/// 配置 Schema
///
/// 包含所有插件的配置定义
///
/// 这个结构体用于解析 spring-rs 生成的 JSON Schema，格式为：
/// ```json
/// {
///   "type": "object",
///   "properties": {
///     "web": { ... },
///     "redis": { ... }
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    /// Schema 类型（通常是 "object"）
    #[serde(rename = "type")]
    pub schema_type: String,

    /// 插件配置映射，键为配置前缀
    /// 在 JSON Schema 中对应 "properties" 字段
    #[serde(rename = "properties")]
    pub plugins: HashMap<String, serde_json::Value>,
}

/// 插件 Schema
///
/// 单个插件的配置定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSchema {
    /// 配置前缀（如 "web"、"redis" 等）
    pub prefix: String,
    /// 配置属性映射，键为属性名
    pub properties: HashMap<String, PropertySchema>,
}

/// 配置属性 Schema
///
/// 单个配置属性的定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    /// 属性名称
    pub name: String,
    /// 类型信息
    pub type_info: TypeInfo,
    /// 属性描述
    pub description: String,
    /// 默认值（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    /// 是否必需
    #[serde(default)]
    pub required: bool,
    /// 废弃信息（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<String>,
    /// 示例代码（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
}

/// 类型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TypeInfo {
    /// 字符串类型
    String {
        /// 枚举值（可选）
        #[serde(skip_serializing_if = "Option::is_none")]
        enum_values: Option<Vec<String>>,
        /// 最小长度（可选）
        #[serde(skip_serializing_if = "Option::is_none")]
        min_length: Option<usize>,
        /// 最大长度（可选）
        #[serde(skip_serializing_if = "Option::is_none")]
        max_length: Option<usize>,
    },
    /// 整数类型
    Integer {
        /// 最小值（可选）
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<i64>,
        /// 最大值（可选）
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<i64>,
    },
    /// 浮点数类型
    Float {
        /// 最小值（可选）
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        /// 最大值（可选）
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
    },
    /// 布尔类型
    Boolean,
    /// 数组类型
    Array {
        /// 元素类型
        item_type: Box<TypeInfo>,
    },
    /// 对象类型（嵌套配置）
    Object {
        /// 嵌套属性
        properties: HashMap<String, PropertySchema>,
    },
}

/// 配置值
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
    /// 字符串值
    String(String),
    /// 整数值
    Integer(i64),
    /// 浮点数值
    Float(f64),
    /// 布尔值
    Boolean(bool),
    /// 数组值
    Array(Vec<Value>),
    /// 对象值
    Table(HashMap<String, Value>),
}

/// Schema 提供者
///
/// 管理配置 Schema，提供配置项元数据查询
#[derive(Clone)]
pub struct SchemaProvider {
    /// Schema 数据（加载后不会改变，直接拥有即可）
    schema: ConfigSchema,
}

impl SchemaProvider {
    /// Schema URL
    const SCHEMA_URL: &'static str = "https://spring-rs.github.io/config-schema.json";

    /// 创建新的 Schema 提供者（使用空 Schema）
    pub fn new() -> Self {
        Self {
            schema: ConfigSchema {
                schema_type: "object".to_string(),
                plugins: HashMap::new(),
            },
        }
    }

    /// 从 URL 加载 Schema
    ///
    /// 如果加载失败，使用内置的备用 Schema
    pub async fn load() -> anyhow::Result<Self> {
        // 尝试从 URL 加载 Schema
        match Self::load_from_url(Self::SCHEMA_URL).await {
            Ok(schema) => {
                tracing::info!("Successfully loaded schema from {}", Self::SCHEMA_URL);
                Ok(Self { schema })
            }
            Err(e) => {
                tracing::warn!("Failed to load schema from URL: {}, using fallback", e);
                // 使用内置备用 Schema
                Ok(Self::with_fallback_schema())
            }
        }
    }

    /// 从指定 URL 加载 Schema
    async fn load_from_url(url: &str) -> anyhow::Result<ConfigSchema> {
        let response = reqwest::get(url).await?;
        let schema = response.json::<ConfigSchema>().await?;
        Ok(schema)
    }

    /// 使用内置备用 Schema
    fn with_fallback_schema() -> Self {
        let fallback_schema = Self::create_fallback_schema();
        Self {
            schema: fallback_schema,
        }
    }

    /// 创建内置备用 Schema
    ///
    /// 包含常见的 spring-rs 插件配置
    fn create_fallback_schema() -> ConfigSchema {
        let mut plugins = HashMap::new();

        // Web 插件配置
        let web_schema = json!({
            "type": "object",
            "properties": {
                "host": {
                    "type": "string",
                    "description": "Web server host address",
                    "default": "0.0.0.0"
                },
                "port": {
                    "type": "integer",
                    "description": "Web server port",
                    "default": 8080,
                    "minimum": 1,
                    "maximum": 65535
                }
            }
        });
        plugins.insert("web".to_string(), web_schema);

        // Redis 插件配置
        let redis_schema = json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "Redis connection URL",
                    "default": "redis://localhost:6379"
                }
            }
        });
        plugins.insert("redis".to_string(), redis_schema);

        ConfigSchema {
            schema_type: "object".to_string(),
            plugins,
        }
    }

    /// 获取插件 Schema
    ///
    /// 检查指定的配置前缀是否在 Schema 中定义
    pub fn has_plugin(&self, prefix: &str) -> bool {
        self.schema.plugins.contains_key(prefix)
    }

    /// 获取所有配置前缀
    ///
    /// 返回所有已注册插件的配置前缀列表
    pub fn get_all_prefixes(&self) -> Vec<String> {
        self.schema.plugins.keys().cloned().collect()
    }
    
    /// 获取插件的 Schema
    ///
    /// 返回指定插件的 JSON Schema
    pub fn get_plugin_schema(&self, prefix: &str) -> Option<&serde_json::Value> {
        self.schema.plugins.get(prefix)
    }

    /// 检查配置属性是否存在
    ///
    /// 查询指定插件的指定属性是否在 Schema 中定义
    pub fn has_property(&self, prefix: &str, property: &str) -> bool {
        if let Some(plugin_schema) = self.schema.plugins.get(prefix) {
            // 尝试解析为 JSON Schema 对象
            if let Some(properties) = plugin_schema.get("properties") {
                if let Some(props_obj) = properties.as_object() {
                    return props_obj.contains_key(property);
                }
            }
        }
        false
    }
}

impl Default for SchemaProvider {
    fn default() -> Self {
        Self::with_fallback_schema()
    }
}

impl SchemaProvider {
    /// 从给定的 ConfigSchema 创建 SchemaProvider（用于测试）
    ///
    /// 这个方法主要用于属性测试，允许使用自定义的 Schema 创建提供者
    pub fn from_schema(schema: ConfigSchema) -> Self {
        Self { schema }
    }
}
