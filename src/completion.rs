//! 智能补全引擎模块

use lsp_types::{CompletionItem, CompletionItemKind, Documentation, MarkupContent, MarkupKind};

use crate::macro_analyzer::SpringMacro;

/// 补全引擎
pub struct CompletionEngine {
    // TODO: 添加字段
}

impl CompletionEngine {
    /// 创建新的补全引擎
    pub fn new() -> Self {
        Self {}
    }

    /// 为宏参数提供补全
    ///
    /// 根据宏的类型提供相应的参数补全项
    ///
    /// # Arguments
    ///
    /// * `macro_info` - 宏信息
    /// * `cursor_position` - 光标位置（用于上下文感知补全）
    ///
    /// # Returns
    ///
    /// 返回补全项列表
    pub fn complete_macro(
        &self,
        macro_info: &SpringMacro,
        _cursor_position: Option<&str>,
    ) -> Vec<CompletionItem> {
        match macro_info {
            SpringMacro::DeriveService(_) => self.complete_service_macro(),
            SpringMacro::Inject(_) => self.complete_inject_macro(),
            SpringMacro::AutoConfig(_) => self.complete_auto_config_macro(),
            SpringMacro::Route(_) => self.complete_route_macro(),
            SpringMacro::Job(_) => self.complete_job_macro(),
        }
    }

    /// 为 Service 宏提供补全
    ///
    /// 提供 inject 属性的参数补全
    fn complete_service_macro(&self) -> Vec<CompletionItem> {
        vec![
            // inject(component) 补全
            CompletionItem {
                label: "inject(component)".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("注入组件".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "从应用上下文中注入已注册的组件实例。\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[inject(component)]\n\
                            db: ConnectPool,\n\
                            ```"
                        .to_string(),
                })),
                insert_text: Some("inject(component)".to_string()),
                ..Default::default()
            },
            // inject(component = "name") 补全
            CompletionItem {
                label: "inject(component = \"name\")".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("注入指定名称的组件".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "使用指定名称从应用上下文中注入组件，适用于多实例场景（如多数据源）。\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[inject(component = \"primary\")]\n\
                            primary_db: ConnectPool,\n\
                            ```"
                        .to_string(),
                })),
                insert_text: Some("inject(component = \"$1\")".to_string()),
                insert_text_format: Some(lsp_types::InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            // inject(config) 补全
            CompletionItem {
                label: "inject(config)".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("注入配置".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "从配置文件中加载配置项。配置项通过 `#[config_prefix]` 指定的前缀从 `config/app.toml` 中读取。\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[inject(config)]\n\
                            config: MyConfig,\n\
                            ```"
                        .to_string(),
                })),
                insert_text: Some("inject(config)".to_string()),
                ..Default::default()
            },
        ]
    }

    /// 为 Inject 宏提供补全
    ///
    /// 提供注入类型的补全（component, config）
    fn complete_inject_macro(&self) -> Vec<CompletionItem> {
        vec![
            // component 补全
            CompletionItem {
                label: "component".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("注入组件".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "从应用上下文中注入已注册的组件实例。\n\n\
                            **使用方式**:\n\
                            - `#[inject(component)]` - 按类型自动查找\n\
                            - `#[inject(component = \"name\")]` - 按名称查找"
                        .to_string(),
                })),
                insert_text: Some("component".to_string()),
                ..Default::default()
            },
            // config 补全
            CompletionItem {
                label: "config".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("注入配置".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "从配置文件中加载配置项。\n\n\
                            **使用方式**:\n\
                            - `#[inject(config)]` - 从 config/app.toml 加载配置"
                        .to_string(),
                })),
                insert_text: Some("config".to_string()),
                ..Default::default()
            },
        ]
    }

    /// 为 AutoConfig 宏提供补全
    ///
    /// 提供常见的配置器类型补全
    fn complete_auto_config_macro(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "WebConfigurator".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Web 路由配置器".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "自动注册 Web 路由处理器。\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[auto_config(WebConfigurator)]\n\
                            #[tokio::main]\n\
                            async fn main() {\n\
                                App::new().add_plugin(WebPlugin).run().await\n\
                            }\n\
                            ```"
                        .to_string(),
                })),
                insert_text: Some("WebConfigurator".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "JobConfigurator".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("任务调度配置器".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "自动注册定时任务。\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[auto_config(JobConfigurator)]\n\
                            #[tokio::main]\n\
                            async fn main() {\n\
                                App::new().add_plugin(JobPlugin).run().await\n\
                            }\n\
                            ```"
                        .to_string(),
                })),
                insert_text: Some("JobConfigurator".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "StreamConfigurator".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("流处理配置器".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "自动注册流消费者。\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[auto_config(StreamConfigurator)]\n\
                            #[tokio::main]\n\
                            async fn main() {\n\
                                App::new().add_plugin(StreamPlugin).run().await\n\
                            }\n\
                            ```"
                        .to_string(),
                })),
                insert_text: Some("StreamConfigurator".to_string()),
                ..Default::default()
            },
        ]
    }

    /// 为路由宏提供补全
    ///
    /// 提供 HTTP 方法和路径参数的补全
    fn complete_route_macro(&self) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // HTTP 方法补全
        let methods = vec![
            ("GET", "获取资源"),
            ("POST", "创建资源"),
            ("PUT", "更新资源（完整）"),
            ("DELETE", "删除资源"),
            ("PATCH", "更新资源（部分）"),
            ("HEAD", "获取资源头信息"),
            ("OPTIONS", "获取支持的方法"),
        ];

        for (method, description) in methods {
            completions.push(CompletionItem {
                label: method.to_string(),
                kind: Some(CompletionItemKind::CONSTANT),
                detail: Some(description.to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "HTTP {} 方法\n\n\
                         **示例**:\n\
                         ```rust\n\
                         #[{}(\"/path\")]\n\
                         async fn handler() -> impl IntoResponse {{\n\
                         }}\n\
                         ```",
                        method,
                        method.to_lowercase()
                    ),
                })),
                insert_text: Some(method.to_string()),
                ..Default::default()
            });
        }

        // 路径参数模板补全
        completions.push(CompletionItem {
            label: "{id}".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("路径参数".to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "路径参数占位符，用于捕获 URL 中的动态部分。\n\n\
                        **示例**:\n\
                        ```rust\n\
                        #[get(\"/users/{id}\")]\n\
                        async fn get_user(Path(id): Path<i64>) -> impl IntoResponse {\n\
                        }\n\
                        ```"
                    .to_string(),
            })),
            insert_text: Some("{${1:id}}".to_string()),
            insert_text_format: Some(lsp_types::InsertTextFormat::SNIPPET),
            ..Default::default()
        });

        completions
    }

    /// 为任务调度宏提供补全
    ///
    /// 提供 cron 表达式、延迟和频率值的补全
    fn complete_job_macro(&self) -> Vec<CompletionItem> {
        vec![
            // Cron 表达式示例
            CompletionItem {
                label: "0 0 * * * *".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("每小时执行".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Cron 表达式：每小时的第 0 分 0 秒执行\n\n\
                            **格式**: 秒 分 时 日 月 星期\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[cron(\"0 0 * * * *\")]\n\
                            async fn hourly_job() {\n\
                            }\n\
                            ```"
                        .to_string(),
                })),
                insert_text: Some("\"0 0 * * * *\"".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "0 0 0 * * *".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("每天午夜执行".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Cron 表达式：每天午夜 00:00:00 执行\n\n\
                            **格式**: 秒 分 时 日 月 星期\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[cron(\"0 0 0 * * *\")]\n\
                            async fn daily_job() {\n\
                            }\n\
                            ```"
                        .to_string(),
                })),
                insert_text: Some("\"0 0 0 * * *\"".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "0 */5 * * * *".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("每 5 分钟执行".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Cron 表达式：每 5 分钟执行一次\n\n\
                            **格式**: 秒 分 时 日 月 星期\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[cron(\"0 */5 * * * *\")]\n\
                            async fn every_five_minutes() {\n\
                            }\n\
                            ```"
                        .to_string(),
                })),
                insert_text: Some("\"0 */5 * * * *\"".to_string()),
                ..Default::default()
            },
            // fix_delay 值示例
            CompletionItem {
                label: "5".to_string(),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some("延迟 5 秒".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "任务完成后延迟 5 秒再次执行\n\n\
                            **示例**:\n\
                            ```rust\n\
                            #[fix_delay(5)]\n\
                            async fn delayed_job() {\n\
                            }\n\
                            ```"
                        .to_string(),
                })),
                insert_text: Some("5".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "10".to_string(),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some("延迟/频率 10 秒".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "延迟或频率为 10 秒\n\n\
                            **fix_delay 示例**:\n\
                            ```rust\n\
                            #[fix_delay(10)]\n\
                            async fn delayed_job() {\n\
                            }\n\
                            ```\n\n\
                            **fix_rate 示例**:\n\
                            ```rust\n\
                            #[fix_rate(10)]\n\
                            async fn periodic_job() {\n\
                            }\n\
                            ```"
                        .to_string(),
                })),
                insert_text: Some("10".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "60".to_string(),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some("延迟/频率 60 秒（1 分钟）".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "延迟或频率为 60 秒（1 分钟）\n\n\
                            **fix_delay 示例**:\n\
                            ```rust\n\
                            #[fix_delay(60)]\n\
                            async fn delayed_job() {\n\
                            }\n\
                            ```\n\n\
                            **fix_rate 示例**:\n\
                            ```rust\n\
                            #[fix_rate(60)]\n\
                            async fn periodic_job() {\n\
                            }\n\
                            ```"
                        .to_string(),
                })),
                insert_text: Some("60".to_string()),
                ..Default::default()
            },
        ]
    }
}

impl Default for CompletionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
