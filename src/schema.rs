//! 配置 Schema 管理模块

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 配置 Schema
///
/// 包含所有插件的配置定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    /// 插件配置映射,键为配置前缀
    pub plugins: HashMap<String, PluginSchema>,
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
/// 管理配置 Schema，提供配置项元数据查询和缓存
#[derive(Clone)]
pub struct SchemaProvider {
    /// Schema 数据（加载后不会改变，直接拥有即可）
    schema: ConfigSchema,
    /// 查询缓存（使用 DashMap 提供并发安全的缓存）
    cache: dashmap::DashMap<String, PluginSchema>,
}

impl SchemaProvider {
    /// Schema URL
    const SCHEMA_URL: &'static str = "https://spring-rs.github.io/config-schema.json";

    /// 创建新的 Schema 提供者（使用空 Schema）
    pub fn new() -> Self {
        Self {
            schema: ConfigSchema {
                plugins: HashMap::new(),
            },
            cache: dashmap::DashMap::new(),
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
                Ok(Self {
                    schema,
                    cache: dashmap::DashMap::new(),
                })
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
            cache: dashmap::DashMap::new(),
        }
    }

    /// 创建内置备用 Schema
    ///
    /// 包含常见的 spring-rs 插件配置
    fn create_fallback_schema() -> ConfigSchema {
        let mut plugins = HashMap::new();

        // Web 插件配置
        let mut web_properties = HashMap::new();
        web_properties.insert(
            "host".to_string(),
            PropertySchema {
                name: "host".to_string(),
                type_info: TypeInfo::String {
                    enum_values: None,
                    min_length: None,
                    max_length: None,
                },
                description: "Web server host address".to_string(),
                default: Some(Value::String("0.0.0.0".to_string())),
                required: false,
                deprecated: None,
                example: Some("host = \"0.0.0.0\"".to_string()),
            },
        );
        web_properties.insert(
            "port".to_string(),
            PropertySchema {
                name: "port".to_string(),
                type_info: TypeInfo::Integer {
                    min: Some(1),
                    max: Some(65535),
                },
                description: "Web server port".to_string(),
                default: Some(Value::Integer(8080)),
                required: false,
                deprecated: None,
                example: Some("port = 8080".to_string()),
            },
        );

        plugins.insert(
            "web".to_string(),
            PluginSchema {
                prefix: "web".to_string(),
                properties: web_properties,
            },
        );

        // Redis 插件配置
        let mut redis_properties = HashMap::new();
        redis_properties.insert(
            "url".to_string(),
            PropertySchema {
                name: "url".to_string(),
                type_info: TypeInfo::String {
                    enum_values: None,
                    min_length: None,
                    max_length: None,
                },
                description: "Redis connection URL".to_string(),
                default: Some(Value::String("redis://localhost:6379".to_string())),
                required: false,
                deprecated: None,
                example: Some("url = \"redis://localhost:6379\"".to_string()),
            },
        );

        plugins.insert(
            "redis".to_string(),
            PluginSchema {
                prefix: "redis".to_string(),
                properties: redis_properties,
            },
        );

        ConfigSchema { plugins }
    }

    /// 获取插件 Schema
    ///
    /// 使用缓存提高性能，返回克隆以避免锁竞争
    pub fn get_plugin_schema(&self, prefix: &str) -> Option<PluginSchema> {
        // 先查缓存（DashMap 并发安全）
        if let Some(cached) = self.cache.get(prefix) {
            return Some(cached.clone());
        }

        // 缓存未命中，从 schema 中查找并缓存
        if let Some(schema) = self.schema.plugins.get(prefix) {
            let cloned = schema.clone();
            self.cache.insert(prefix.to_string(), cloned.clone());
            Some(cloned)
        } else {
            None
        }
    }

    /// 获取属性 Schema
    ///
    /// 查询指定插件的指定属性
    pub fn get_property_schema(&self, prefix: &str, property: &str) -> Option<PropertySchema> {
        let plugin_schema = self.get_plugin_schema(prefix)?;
        plugin_schema.properties.get(property).cloned()
    }

    /// 获取所有配置前缀
    ///
    /// 返回所有已注册插件的配置前缀列表
    pub fn get_all_prefixes(&self) -> Vec<String> {
        self.schema.plugins.keys().cloned().collect()
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
        Self {
            schema,
            cache: dashmap::DashMap::new(),
        }
    }
}
