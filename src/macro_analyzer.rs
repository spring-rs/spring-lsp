//! Rust 宏分析模块

use lsp_types::{Range, Url};
use syn::__private::Span;
use syn::spanned::Spanned;

/// Rust 文档模型
#[derive(Debug, Clone)]
pub struct RustDocument {
    /// 文档 URI
    pub uri: Url,
    /// 文档内容
    pub content: String,
    /// 提取的 spring-rs 宏
    pub macros: Vec<SpringMacro>,
}

/// Spring-rs 宏枚举
#[derive(Debug, Clone)]
pub enum SpringMacro {
    /// Service 派生宏
    DeriveService(ServiceMacro),
    /// Inject 属性宏
    Inject(InjectMacro),
    /// AutoConfig 属性宏
    AutoConfig(AutoConfigMacro),
    /// 路由宏
    Route(RouteMacro),
    /// 任务调度宏
    Job(JobMacro),
}

/// Service 派生宏信息
#[derive(Debug, Clone)]
pub struct ServiceMacro {
    /// 结构体名称
    pub struct_name: String,
    /// 字段列表
    pub fields: Vec<Field>,
    /// 宏在源代码中的位置
    pub range: Range,
}

/// 字段信息
#[derive(Debug, Clone)]
pub struct Field {
    /// 字段名称
    pub name: String,
    /// 字段类型名称
    pub type_name: String,
    /// 注入宏（如果有）
    pub inject: Option<InjectMacro>,
}

/// Inject 属性宏信息
#[derive(Debug, Clone)]
pub struct InjectMacro {
    /// 注入类型
    pub inject_type: InjectType,
    /// 组件名称（可选）
    pub component_name: Option<String>,
    /// 宏在源代码中的位置
    pub range: Range,
}

/// 注入类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InjectType {
    /// 注入组件
    Component,
    /// 注入配置
    Config,
}

/// AutoConfig 属性宏信息
#[derive(Debug, Clone)]
pub struct AutoConfigMacro {
    /// 配置器类型
    pub configurator_type: String,
    /// 宏在源代码中的位置
    pub range: Range,
}

/// 路由宏信息
#[derive(Debug, Clone)]
pub struct RouteMacro {
    /// 路由路径
    pub path: String,
    /// HTTP 方法列表
    pub methods: Vec<HttpMethod>,
    /// 中间件列表
    pub middlewares: Vec<String>,
    /// 处理器函数名称
    pub handler_name: String,
    /// 宏在源代码中的位置
    pub range: Range,
}

/// HTTP 方法
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
    Connect,
    Trace,
}

impl HttpMethod {
    /// 从字符串解析 HTTP 方法
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(HttpMethod::Get),
            "POST" => Some(HttpMethod::Post),
            "PUT" => Some(HttpMethod::Put),
            "DELETE" => Some(HttpMethod::Delete),
            "PATCH" => Some(HttpMethod::Patch),
            "HEAD" => Some(HttpMethod::Head),
            "OPTIONS" => Some(HttpMethod::Options),
            "CONNECT" => Some(HttpMethod::Connect),
            "TRACE" => Some(HttpMethod::Trace),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Connect => "CONNECT",
            HttpMethod::Trace => "TRACE",
        }
    }
}

/// 任务调度宏信息
#[derive(Debug, Clone)]
pub enum JobMacro {
    /// Cron 表达式任务
    Cron {
        /// Cron 表达式
        expression: String,
        /// 宏在源代码中的位置
        range: Range,
    },
    /// 固定延迟任务
    FixDelay {
        /// 延迟秒数
        seconds: u64,
        /// 宏在源代码中的位置
        range: Range,
    },
    /// 固定频率任务
    FixRate {
        /// 频率秒数
        seconds: u64,
        /// 宏在源代码中的位置
        range: Range,
    },
}

/// 宏分析器
pub struct MacroAnalyzer;

impl MacroAnalyzer {
    /// 创建新的宏分析器
    pub fn new() -> Self {
        Self
    }

    /// 为宏提供悬停提示
    ///
    /// 当用户悬停在 spring-rs 宏上时，显示宏的详细信息和展开后的代码
    ///
    /// # Arguments
    ///
    /// * `macro_info` - 要显示悬停提示的宏信息
    ///
    /// # Returns
    ///
    /// 返回格式化的悬停提示内容（Markdown 格式）
    pub fn hover_macro(&self, macro_info: &SpringMacro) -> String {
        match macro_info {
            SpringMacro::DeriveService(service) => self.hover_service_macro(service),
            SpringMacro::Inject(inject) => self.hover_inject_macro(inject),
            SpringMacro::AutoConfig(auto_config) => self.hover_auto_config_macro(auto_config),
            SpringMacro::Route(route) => self.hover_route_macro(route),
            SpringMacro::Job(job) => self.hover_job_macro(job),
        }
    }

    /// 为 Service 宏提供悬停提示
    ///
    /// 显示 Service 宏的说明和生成的 trait 实现代码
    fn hover_service_macro(&self, service: &ServiceMacro) -> String {
        let mut hover = String::new();

        // 添加标题
        hover.push_str("# Service 派生宏\n\n");

        // 添加说明
        hover.push_str("自动为结构体实现依赖注入功能，从应用上下文中获取组件和配置。\n\n");

        // 添加结构体信息
        hover.push_str(&format!("**结构体**: `{}`\n\n", service.struct_name));

        // 添加字段信息
        if !service.fields.is_empty() {
            hover.push_str("**注入字段**:\n\n");
            for field in &service.fields {
                hover.push_str(&format!("- `{}`: `{}`", field.name, field.type_name));
                if let Some(inject) = &field.inject {
                    match inject.inject_type {
                        InjectType::Component => {
                            if let Some(name) = &inject.component_name {
                                hover.push_str(&format!(" - 注入组件 `\"{}\"`", name));
                            } else {
                                hover.push_str(" - 注入组件");
                            }
                        }
                        InjectType::Config => {
                            hover.push_str(" - 注入配置");
                        }
                    }
                }
                hover.push_str("\n");
            }
            hover.push_str("\n");
        }

        // 添加展开后的代码
        hover.push_str("**展开后的代码**:\n\n");
        hover.push_str("```rust\n");
        hover.push_str(&self.expand_service_macro(service));
        hover.push_str("```\n");

        hover
    }

    /// 为 Inject 属性提供悬停提示
    ///
    /// 显示注入的组件类型和来源信息
    fn hover_inject_macro(&self, inject: &InjectMacro) -> String {
        let mut hover = String::new();

        // 添加标题
        hover.push_str("# Inject 属性宏\n\n");

        // 添加说明
        hover.push_str("标记字段从应用上下文中自动注入依赖。\n\n");

        // 添加注入类型信息
        match inject.inject_type {
            InjectType::Component => {
                hover.push_str("**注入类型**: 组件 (Component)\n\n");
                hover.push_str("从应用上下文中获取已注册的组件实例。\n\n");

                if let Some(name) = &inject.component_name {
                    hover.push_str(&format!("**组件名称**: `\"{}\"`\n\n", name));
                    hover.push_str("使用指定名称查找组件，适用于多实例场景（如多数据源）。\n\n");
                    hover.push_str("**注入代码**:\n\n");
                    hover.push_str("```rust\n");
                    hover.push_str(&format!("app.get_component::<T>(\"{}\")\n", name));
                    hover.push_str("```\n");
                } else {
                    hover.push_str("使用类型自动查找组件。\n\n");
                    hover.push_str("**注入代码**:\n\n");
                    hover.push_str("```rust\n");
                    hover.push_str("app.get_component::<T>()\n");
                    hover.push_str("```\n");
                }
            }
            InjectType::Config => {
                hover.push_str("**注入类型**: 配置 (Config)\n\n");
                hover.push_str("从配置文件中加载配置项。\n\n");
                hover.push_str(
                    "配置项通过 `#[config_prefix]` 指定的前缀从 `config/app.toml` 中读取。\n\n",
                );
                hover.push_str("**注入代码**:\n\n");
                hover.push_str("```rust\n");
                hover.push_str("app.get_config::<T>()\n");
                hover.push_str("```\n");
            }
        }

        // 添加示例
        hover.push_str("\n**使用示例**:\n\n");
        hover.push_str("```rust\n");
        hover.push_str("#[derive(Clone, Service)]\n");
        hover.push_str("struct MyService {\n");

        match inject.inject_type {
            InjectType::Component => {
                if let Some(name) = &inject.component_name {
                    hover.push_str(&format!("    #[inject(component = \"{}\")]\n", name));
                    hover.push_str("    db: ConnectPool,\n");
                } else {
                    hover.push_str("    #[inject(component)]\n");
                    hover.push_str("    db: ConnectPool,\n");
                }
            }
            InjectType::Config => {
                hover.push_str("    #[inject(config)]\n");
                hover.push_str("    config: MyConfig,\n");
            }
        }

        hover.push_str("}\n");
        hover.push_str("```\n");

        hover
    }

    /// 为 AutoConfig 宏提供悬停提示
    fn hover_auto_config_macro(&self, auto_config: &AutoConfigMacro) -> String {
        let mut hover = String::new();

        hover.push_str("# AutoConfig 属性宏\n\n");
        hover.push_str("自动注册配置器，在应用启动时配置路由、任务等。\n\n");
        hover.push_str(&format!(
            "**配置器类型**: `{}`\n\n",
            auto_config.configurator_type
        ));
        hover.push_str("**展开后的代码**:\n\n");
        hover.push_str("```rust\n");
        hover.push_str(&self.expand_auto_config_macro(auto_config));
        hover.push_str("```\n");

        hover
    }

    /// 为路由宏提供悬停提示
    fn hover_route_macro(&self, route: &RouteMacro) -> String {
        let mut hover = String::new();

        hover.push_str("# 路由宏\n\n");
        hover.push_str("注册 HTTP 路由处理器。\n\n");
        hover.push_str(&format!("**路由路径**: `{}`\n\n", route.path));
        hover.push_str(&format!(
            "**HTTP 方法**: {}\n\n",
            route
                .methods
                .iter()
                .map(|m| format!("`{}`", m.as_str()))
                .collect::<Vec<_>>()
                .join(", ")
        ));

        if !route.middlewares.is_empty() {
            hover.push_str(&format!(
                "**中间件**: {}\n\n",
                route
                    .middlewares
                    .iter()
                    .map(|m| format!("`{}`", m))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        hover.push_str(&format!("**处理器函数**: `{}`\n\n", route.handler_name));
        hover.push_str("**展开后的代码**:\n\n");
        hover.push_str("```rust\n");
        hover.push_str(&self.expand_route_macro(route));
        hover.push_str("```\n");

        hover
    }

    /// 为任务调度宏提供悬停提示
    fn hover_job_macro(&self, job: &JobMacro) -> String {
        let mut hover = String::new();

        hover.push_str("# 任务调度宏\n\n");

        match job {
            JobMacro::Cron { expression, .. } => {
                hover.push_str("定时任务，使用 Cron 表达式指定执行时间。\n\n");
                hover.push_str(&format!("**Cron 表达式**: `{}`\n\n", expression));
                hover.push_str("**格式**: `秒 分 时 日 月 星期`\n\n");
            }
            JobMacro::FixDelay { seconds, .. } => {
                hover.push_str("固定延迟任务，任务完成后延迟指定秒数再次执行。\n\n");
                hover.push_str(&format!("**延迟秒数**: `{}`\n\n", seconds));
            }
            JobMacro::FixRate { seconds, .. } => {
                hover.push_str("固定频率任务，每隔指定秒数执行一次。\n\n");
                hover.push_str(&format!("**频率秒数**: `{}`\n\n", seconds));
            }
        }

        hover.push_str("**展开后的代码**:\n\n");
        hover.push_str("```rust\n");
        hover.push_str(&self.expand_job_macro(job));
        hover.push_str("```\n");

        hover
    }

    /// 展开宏，生成展开后的代码
    ///
    /// 为 spring-rs 宏生成展开后的 Rust 代码，帮助开发者理解宏的实际效果
    ///
    /// # Arguments
    ///
    /// * `macro_info` - 要展开的宏信息
    ///
    /// # Returns
    ///
    /// 返回展开后的 Rust 代码字符串
    pub fn expand_macro(&self, macro_info: &SpringMacro) -> String {
        match macro_info {
            SpringMacro::DeriveService(service) => self.expand_service_macro(service),
            SpringMacro::Inject(inject) => self.expand_inject_macro(inject),
            SpringMacro::AutoConfig(auto_config) => self.expand_auto_config_macro(auto_config),
            SpringMacro::Route(route) => self.expand_route_macro(route),
            SpringMacro::Job(job) => self.expand_job_macro(job),
        }
    }

    /// 展开 Service 派生宏
    ///
    /// 生成 Service trait 的实现代码，包括依赖注入逻辑
    fn expand_service_macro(&self, service: &ServiceMacro) -> String {
        let struct_name = &service.struct_name;
        let mut code = String::new();

        // 生成原始结构体定义（带注释）
        code.push_str(&format!("// 原始定义\n"));
        code.push_str(&format!("#[derive(Clone)]\n"));
        code.push_str(&format!("pub struct {} {{\n", struct_name));
        for field in &service.fields {
            if let Some(inject) = &field.inject {
                let inject_type = match inject.inject_type {
                    InjectType::Component => "component",
                    InjectType::Config => "config",
                };
                if let Some(name) = &inject.component_name {
                    code.push_str(&format!("    #[inject({} = \"{}\")]\n", inject_type, name));
                } else {
                    code.push_str(&format!("    #[inject({})]\n", inject_type));
                }
            }
            code.push_str(&format!("    pub {}: {},\n", field.name, field.type_name));
        }
        code.push_str("}\n\n");

        // 生成 Service trait 实现
        code.push_str(&format!("// 展开后的代码\n"));
        code.push_str(&format!("impl {} {{\n", struct_name));
        code.push_str("    /// 从应用上下文构建服务实例\n");
        code.push_str("    pub fn build(app: &AppBuilder) -> Result<Self> {\n");

        // 为每个字段生成注入代码
        for field in &service.fields {
            if let Some(inject) = &field.inject {
                match inject.inject_type {
                    InjectType::Component => {
                        if let Some(name) = &inject.component_name {
                            code.push_str(&format!(
                                "        let {} = app.get_component::<{}>(\"{}\")?\n",
                                field.name, field.type_name, name
                            ));
                        } else {
                            code.push_str(&format!(
                                "        let {} = app.get_component::<{}>()?;\n",
                                field.name, field.type_name
                            ));
                        }
                    }
                    InjectType::Config => {
                        code.push_str(&format!(
                            "        let {} = app.get_config::<{}>()?;\n",
                            field.name, field.type_name
                        ));
                    }
                }
            } else {
                // 没有 inject 属性的字段需要手动初始化
                code.push_str(&format!(
                    "        let {} = Default::default(); // 需要手动初始化\n",
                    field.name
                ));
            }
        }

        code.push_str("\n        Ok(Self {\n");
        for field in &service.fields {
            code.push_str(&format!("            {},\n", field.name));
        }
        code.push_str("        })\n");
        code.push_str("    }\n");
        code.push_str("}\n");

        code
    }

    /// 展开 Inject 属性宏
    ///
    /// 生成注入字段的说明
    fn expand_inject_macro(&self, inject: &InjectMacro) -> String {
        let mut code = String::new();

        code.push_str("// Inject 属性展开\n");
        code.push_str("// 这个字段将在运行时从应用上下文中注入\n");

        match inject.inject_type {
            InjectType::Component => {
                if let Some(name) = &inject.component_name {
                    code.push_str(&format!("// 注入类型: 组件\n"));
                    code.push_str(&format!("// 组件名称: \"{}\"\n", name));
                    code.push_str(&format!(
                        "// 注入代码: app.get_component::<T>(\"{}\")\n",
                        name
                    ));
                } else {
                    code.push_str(&format!("// 注入类型: 组件\n"));
                    code.push_str(&format!("// 注入代码: app.get_component::<T>()\n"));
                }
            }
            InjectType::Config => {
                code.push_str(&format!("// 注入类型: 配置\n"));
                code.push_str(&format!("// 注入代码: app.get_config::<T>()\n"));
            }
        }

        code
    }

    /// 展开 AutoConfig 宏
    ///
    /// 生成自动配置的说明
    fn expand_auto_config_macro(&self, auto_config: &AutoConfigMacro) -> String {
        let mut code = String::new();

        code.push_str("// AutoConfig 宏展开\n");
        code.push_str(&format!(
            "// 配置器类型: {}\n",
            auto_config.configurator_type
        ));
        code.push_str("// 这个函数将在应用启动时自动注册配置\n");
        code.push_str("// 展开后的代码:\n");
        code.push_str("// \n");
        code.push_str("// fn main() {\n");
        code.push_str(&format!(
            "//     let configurator = {}::new();\n",
            auto_config.configurator_type
        ));
        code.push_str("//     configurator.configure(&mut app);\n");
        code.push_str("//     // ... 原函数体\n");
        code.push_str("// }\n");

        code
    }

    /// 展开路由宏
    ///
    /// 生成路由注册代码
    fn expand_route_macro(&self, route: &RouteMacro) -> String {
        let mut code = String::new();

        code.push_str("// 路由宏展开\n");
        code.push_str(&format!("// 路由路径: {}\n", route.path));
        code.push_str(&format!(
            "// HTTP 方法: {}\n",
            route
                .methods
                .iter()
                .map(|m| m.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));

        if !route.middlewares.is_empty() {
            code.push_str(&format!("// 中间件: {}\n", route.middlewares.join(", ")));
        }

        code.push_str("// \n");
        code.push_str("// 展开后的代码:\n");
        code.push_str("// \n");

        for method in &route.methods {
            code.push_str(&format!(
                "// router.route(\"{}\", {}, {});\n",
                route.path,
                method.as_str().to_lowercase(),
                route.handler_name
            ));
        }

        if !route.middlewares.is_empty() {
            code.push_str("// \n");
            code.push_str("// 应用中间件:\n");
            for middleware in &route.middlewares {
                code.push_str(&format!("// .layer({})\n", middleware));
            }
        }

        code
    }

    /// 展开任务调度宏
    ///
    /// 生成任务调度注册代码
    fn expand_job_macro(&self, job: &JobMacro) -> String {
        let mut code = String::new();

        code.push_str("// 任务调度宏展开\n");

        match job {
            JobMacro::Cron { expression, .. } => {
                code.push_str(&format!("// 任务类型: Cron\n"));
                code.push_str(&format!("// Cron 表达式: {}\n", expression));
                code.push_str("// \n");
                code.push_str("// 展开后的代码:\n");
                code.push_str("// \n");
                code.push_str(&format!("// scheduler.add_job(\n"));
                code.push_str(&format!(
                    "//     CronJob::new(\"{}\", || async {{\n",
                    expression
                ));
                code.push_str("//         // 任务函数体\n");
                code.push_str("//     }})\n");
                code.push_str("// );\n");
            }
            JobMacro::FixDelay { seconds, .. } => {
                code.push_str(&format!("// 任务类型: FixDelay\n"));
                code.push_str(&format!("// 延迟秒数: {}\n", seconds));
                code.push_str("// 说明: 任务完成后延迟指定秒数再次执行\n");
                code.push_str("// \n");
                code.push_str("// 展开后的代码:\n");
                code.push_str("// \n");
                code.push_str(&format!("// scheduler.add_job(\n"));
                code.push_str(&format!(
                    "//     FixDelayJob::new({}, || async {{\n",
                    seconds
                ));
                code.push_str("//         // 任务函数体\n");
                code.push_str("//     }})\n");
                code.push_str("// );\n");
            }
            JobMacro::FixRate { seconds, .. } => {
                code.push_str(&format!("// 任务类型: FixRate\n"));
                code.push_str(&format!("// 频率秒数: {}\n", seconds));
                code.push_str("// 说明: 每隔指定秒数执行一次任务\n");
                code.push_str("// \n");
                code.push_str("// 展开后的代码:\n");
                code.push_str("// \n");
                code.push_str(&format!("// scheduler.add_job(\n"));
                code.push_str(&format!(
                    "//     FixRateJob::new({}, || async {{\n",
                    seconds
                ));
                code.push_str("//         // 任务函数体\n");
                code.push_str("//     }})\n");
                code.push_str("// );\n");
            }
        }

        code
    }

    /// 解析 Rust 源代码
    ///
    /// 使用 syn crate 解析 Rust 代码为语法树
    ///
    /// # Arguments
    ///
    /// * `uri` - 文档 URI
    /// * `content` - Rust 源代码内容
    ///
    /// # Returns
    ///
    /// 返回解析后的 RustDocument，如果解析失败则返回错误
    pub fn parse(&self, uri: Url, content: String) -> Result<RustDocument, syn::Error> {
        // 使用 syn 解析 Rust 代码
        let _syntax_tree = syn::parse_file(&content)?;

        // 创建 RustDocument
        // 注意：实际的宏提取将在 extract_macros 中完成
        let doc = RustDocument {
            uri,
            content,
            macros: Vec::new(),
        };

        Ok(doc)
    }

    /// 从 RustDocument 中提取 spring-rs 宏
    ///
    /// 遍历语法树，识别并提取所有 spring-rs 特定的宏
    ///
    /// # Arguments
    ///
    /// * `doc` - 已解析的 RustDocument
    ///
    /// # Returns
    ///
    /// 返回包含提取的宏的新 RustDocument
    pub fn extract_macros(&self, mut doc: RustDocument) -> Result<RustDocument, syn::Error> {
        // 重新解析内容以获取语法树
        let syntax_tree = syn::parse_file(&doc.content)?;

        let mut macros = Vec::new();

        // 遍历所有项（items）
        for item in &syntax_tree.items {
            match item {
                // 处理结构体定义
                syn::Item::Struct(item_struct) => {
                    // 检查是否有 #[derive(Service)]
                    if let Some(service_macro) = self.extract_service_macro(item_struct) {
                        macros.push(SpringMacro::DeriveService(service_macro));
                    }
                }
                // 处理函数定义
                syn::Item::Fn(item_fn) => {
                    // 检查路由宏
                    if let Some(route_macro) = self.extract_route_macro(item_fn) {
                        macros.push(SpringMacro::Route(route_macro));
                    }

                    // 检查 AutoConfig 宏
                    if let Some(auto_config_macro) = self.extract_auto_config_macro(item_fn) {
                        macros.push(SpringMacro::AutoConfig(auto_config_macro));
                    }

                    // 检查任务调度宏
                    if let Some(job_macro) = self.extract_job_macro(item_fn) {
                        macros.push(SpringMacro::Job(job_macro));
                    }
                }
                _ => {}
            }
        }

        doc.macros = macros;
        Ok(doc)
    }

    /// 提取 Service 派生宏
    fn extract_service_macro(&self, item_struct: &syn::ItemStruct) -> Option<ServiceMacro> {
        // 检查是否有 #[derive(...)] 属性
        for attr in &item_struct.attrs {
            if attr.path().is_ident("derive") {
                // 解析 derive 属性的内容
                if let Ok(meta_list) = attr.meta.require_list() {
                    // 检查是否包含 Service
                    let has_service = meta_list.tokens.to_string().contains("Service");

                    if has_service {
                        // 提取字段信息
                        let fields = self.extract_fields(&item_struct.fields);

                        return Some(ServiceMacro {
                            struct_name: item_struct.ident.to_string(),
                            fields,
                            range: self.span_to_range(&item_struct.ident.span()),
                        });
                    }
                }
            }
        }
        None
    }

    /// 提取结构体字段信息
    fn extract_fields(&self, fields: &syn::Fields) -> Vec<Field> {
        let mut result = Vec::new();

        match fields {
            syn::Fields::Named(fields_named) => {
                for field in &fields_named.named {
                    if let Some(ident) = &field.ident {
                        let inject = self.extract_inject_macro(&field.attrs);

                        result.push(Field {
                            name: ident.to_string(),
                            type_name: self.type_to_string(&field.ty),
                            inject,
                        });
                    }
                }
            }
            _ => {}
        }

        result
    }

    /// 提取 Inject 属性宏
    fn extract_inject_macro(&self, attrs: &[syn::Attribute]) -> Option<InjectMacro> {
        for attr in attrs {
            if attr.path().is_ident("inject") {
                // 解析 inject 属性的参数
                if let Ok(meta_list) = attr.meta.require_list() {
                    let tokens_str = meta_list.tokens.to_string();

                    // 判断注入类型
                    let inject_type = if tokens_str.contains("component") {
                        InjectType::Component
                    } else if tokens_str.contains("config") {
                        InjectType::Config
                    } else {
                        continue;
                    };

                    // 提取组件名称（如果有）
                    let component_name = self.extract_component_name(&tokens_str);

                    return Some(InjectMacro {
                        inject_type,
                        component_name,
                        range: self.span_to_range(&attr.span()),
                    });
                }
            }
        }
        None
    }

    /// 从 inject 属性参数中提取组件名称
    fn extract_component_name(&self, tokens_str: &str) -> Option<String> {
        // 查找 component = "name" 或 component = name 模式
        if let Some(eq_pos) = tokens_str.find('=') {
            let after_eq = &tokens_str[eq_pos + 1..].trim();
            // 移除引号
            let name = after_eq.trim_matches('"').trim();
            if !name.is_empty() && name != "component" && name != "config" {
                return Some(name.to_string());
            }
        }
        None
    }

    /// 提取路由宏
    fn extract_route_macro(&self, item_fn: &syn::ItemFn) -> Option<RouteMacro> {
        for attr in &item_fn.attrs {
            // 检查各种路由宏
            let method_and_path: Option<(Vec<HttpMethod>, String)> = if attr.path().is_ident("get")
            {
                self.extract_path_from_attr(attr)
                    .map(|path| (vec![HttpMethod::Get], path))
            } else if attr.path().is_ident("post") {
                self.extract_path_from_attr(attr)
                    .map(|path| (vec![HttpMethod::Post], path))
            } else if attr.path().is_ident("put") {
                self.extract_path_from_attr(attr)
                    .map(|path| (vec![HttpMethod::Put], path))
            } else if attr.path().is_ident("delete") {
                self.extract_path_from_attr(attr)
                    .map(|path| (vec![HttpMethod::Delete], path))
            } else if attr.path().is_ident("patch") {
                self.extract_path_from_attr(attr)
                    .map(|path| (vec![HttpMethod::Patch], path))
            } else if attr.path().is_ident("head") {
                self.extract_path_from_attr(attr)
                    .map(|path| (vec![HttpMethod::Head], path))
            } else if attr.path().is_ident("options") {
                self.extract_path_from_attr(attr)
                    .map(|path| (vec![HttpMethod::Options], path))
            } else if attr.path().is_ident("route") {
                // route 宏可以指定多个方法
                self.extract_route_attr(attr)
            } else {
                None
            };

            if let Some((methods, path)) = method_and_path {
                // 提取中间件（如果有）
                let middlewares = self.extract_middlewares(&item_fn.attrs);

                return Some(RouteMacro {
                    path,
                    methods,
                    middlewares,
                    handler_name: item_fn.sig.ident.to_string(),
                    range: self.span_to_range(&item_fn.sig.ident.span()),
                });
            }
        }
        None
    }

    /// 从属性中提取路径
    fn extract_path_from_attr(&self, attr: &syn::Attribute) -> Option<String> {
        // 解析属性参数，期望是字符串字面量
        if let Ok(meta_list) = attr.meta.require_list() {
            let tokens_str = meta_list.tokens.to_string();
            // 移除引号
            let path = tokens_str.trim().trim_matches('"');
            return Some(path.to_string());
        }
        None
    }

    /// 从 route 属性中提取路径和方法
    fn extract_route_attr(&self, attr: &syn::Attribute) -> Option<(Vec<HttpMethod>, String)> {
        if let Ok(meta_list) = attr.meta.require_list() {
            let tokens_str = meta_list.tokens.to_string();

            // 提取路径（第一个字符串字面量）
            let path = if let Some(start) = tokens_str.find('"') {
                if let Some(end) = tokens_str[start + 1..].find('"') {
                    tokens_str[start + 1..start + 1 + end].to_string()
                } else {
                    return None;
                }
            } else {
                return None;
            };

            // 提取方法（method = "GET" 或 method = "POST" 等）
            let mut methods = Vec::new();
            for part in tokens_str.split(',') {
                if part.contains("method") {
                    if let Some(eq_pos) = part.find('=') {
                        let method_str = part[eq_pos + 1..].trim().trim_matches('"');
                        if let Some(method) = HttpMethod::from_str(method_str) {
                            methods.push(method);
                        }
                    }
                }
            }

            if !methods.is_empty() {
                return Some((methods, path));
            }
        }
        None
    }

    /// 提取中间件列表
    fn extract_middlewares(&self, attrs: &[syn::Attribute]) -> Vec<String> {
        let mut middlewares = Vec::new();

        for attr in attrs {
            if attr.path().is_ident("middlewares") {
                if let Ok(meta_list) = attr.meta.require_list() {
                    let tokens_str = meta_list.tokens.to_string();
                    // 简单解析，按逗号分割
                    for part in tokens_str.split(',') {
                        let middleware = part.trim().to_string();
                        if !middleware.is_empty() {
                            middlewares.push(middleware);
                        }
                    }
                }
            }
        }

        middlewares
    }

    /// 提取 AutoConfig 宏
    fn extract_auto_config_macro(&self, item_fn: &syn::ItemFn) -> Option<AutoConfigMacro> {
        for attr in &item_fn.attrs {
            if attr.path().is_ident("auto_config") {
                // 提取配置器类型
                let configurator_type = if let Ok(meta_list) = attr.meta.require_list() {
                    meta_list.tokens.to_string()
                } else {
                    String::new()
                };

                return Some(AutoConfigMacro {
                    configurator_type,
                    range: self.span_to_range(&attr.span()),
                });
            }
        }
        None
    }

    /// 提取任务调度宏
    fn extract_job_macro(&self, item_fn: &syn::ItemFn) -> Option<JobMacro> {
        for attr in &item_fn.attrs {
            if attr.path().is_ident("cron") {
                // 提取 cron 表达式
                if let Some(expression) = self.extract_path_from_attr(attr) {
                    return Some(JobMacro::Cron {
                        expression,
                        range: self.span_to_range(&attr.span()),
                    });
                }
            } else if attr.path().is_ident("fix_delay") {
                // 提取延迟秒数
                if let Ok(meta_list) = attr.meta.require_list() {
                    let tokens_str = meta_list.tokens.to_string();
                    if let Ok(seconds) = tokens_str.trim().parse::<u64>() {
                        return Some(JobMacro::FixDelay {
                            seconds,
                            range: self.span_to_range(&attr.span()),
                        });
                    }
                }
            } else if attr.path().is_ident("fix_rate") {
                // 提取频率秒数
                if let Ok(meta_list) = attr.meta.require_list() {
                    let tokens_str = meta_list.tokens.to_string();
                    if let Ok(seconds) = tokens_str.trim().parse::<u64>() {
                        return Some(JobMacro::FixRate {
                            seconds,
                            range: self.span_to_range(&attr.span()),
                        });
                    }
                }
            }
        }
        None
    }

    /// 将类型转换为字符串
    fn type_to_string(&self, ty: &syn::Type) -> String {
        match ty {
            syn::Type::Path(type_path) => type_path
                .path
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::"),
            _ => "Unknown".to_string(),
        }
    }

    /// 将 Span 转换为 LSP Range
    ///
    /// 注意：当前实现返回一个默认的 Range，因为 proc_macro2::Span 在非 proc-macro 上下文中
    /// 无法获取准确的位置信息。在实际的 LSP 服务器中，我们会使用文档的行列信息。
    fn span_to_range(&self, _span: &Span) -> Range {
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

    /// 验证宏参数的正确性
    ///
    /// 检查宏参数是否符合 spring-rs 的要求，生成错误诊断和修复建议
    ///
    /// # Arguments
    ///
    /// * `macro_info` - 要验证的宏信息
    ///
    /// # Returns
    ///
    /// 返回诊断信息列表，如果没有错误则返回空列表
    pub fn validate_macro(&self, macro_info: &SpringMacro) -> Vec<lsp_types::Diagnostic> {
        match macro_info {
            SpringMacro::DeriveService(service) => self.validate_service_macro(service),
            SpringMacro::Inject(inject) => self.validate_inject_macro(inject),
            SpringMacro::AutoConfig(auto_config) => self.validate_auto_config_macro(auto_config),
            SpringMacro::Route(route) => self.validate_route_macro(route),
            SpringMacro::Job(job) => self.validate_job_macro(job),
        }
    }

    /// 验证 Service 宏
    ///
    /// 检查结构体字段的 inject 属性是否正确
    fn validate_service_macro(&self, service: &ServiceMacro) -> Vec<lsp_types::Diagnostic> {
        let mut diagnostics = Vec::new();

        // 检查每个字段的 inject 属性
        for field in &service.fields {
            if let Some(inject) = &field.inject {
                // 验证 inject 属性
                let inject_diagnostics = self.validate_inject_macro(inject);
                diagnostics.extend(inject_diagnostics);

                // 检查组件名称是否为空字符串
                if let Some(name) = &inject.component_name {
                    if name.is_empty() {
                        diagnostics.push(lsp_types::Diagnostic {
                            range: inject.range,
                            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                            code: Some(lsp_types::NumberOrString::String("E001".to_string())),
                            source: Some("spring-lsp".to_string()),
                            message: format!("字段 '{}' 的组件名称不能为空字符串", field.name),
                            related_information: None,
                            tags: None,
                            code_description: None,
                            data: None,
                        });
                    }
                }
            }
        }

        diagnostics
    }

    /// 验证 Inject 宏
    ///
    /// 检查注入类型和组件名称是否有效
    fn validate_inject_macro(&self, inject: &InjectMacro) -> Vec<lsp_types::Diagnostic> {
        let mut diagnostics = Vec::new();

        // 检查 config 类型的注入不应该有组件名称
        if inject.inject_type == InjectType::Config && inject.component_name.is_some() {
            diagnostics.push(lsp_types::Diagnostic {
                range: inject.range,
                severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                code: Some(lsp_types::NumberOrString::String("E002".to_string())),
                source: Some("spring-lsp".to_string()),
                message: "配置注入 (config) 不应该指定组件名称".to_string(),
                related_information: None,
                tags: None,
                code_description: None,
                data: None,
            });
        }

        diagnostics
    }

    /// 验证 AutoConfig 宏
    ///
    /// 检查配置器类型是否有效
    fn validate_auto_config_macro(
        &self,
        auto_config: &AutoConfigMacro,
    ) -> Vec<lsp_types::Diagnostic> {
        let mut diagnostics = Vec::new();

        // 检查配置器类型是否为空
        if auto_config.configurator_type.is_empty() {
            diagnostics.push(lsp_types::Diagnostic {
                range: auto_config.range,
                severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                code: Some(lsp_types::NumberOrString::String("E003".to_string())),
                source: Some("spring-lsp".to_string()),
                message: "AutoConfig 宏必须指定配置器类型".to_string(),
                related_information: None,
                tags: None,
                code_description: None,
                data: None,
            });
        }

        diagnostics
    }

    /// 验证路由宏
    ///
    /// 检查路径格式和 HTTP 方法是否有效
    fn validate_route_macro(&self, route: &RouteMacro) -> Vec<lsp_types::Diagnostic> {
        let mut diagnostics = Vec::new();

        // 检查路径是否为空
        if route.path.is_empty() {
            diagnostics.push(lsp_types::Diagnostic {
                range: route.range,
                severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                code: Some(lsp_types::NumberOrString::String("E004".to_string())),
                source: Some("spring-lsp".to_string()),
                message: "路由路径不能为空".to_string(),
                related_information: None,
                tags: None,
                code_description: None,
                data: None,
            });
        } else {
            // 检查路径是否以 / 开头
            if !route.path.starts_with('/') {
                diagnostics.push(lsp_types::Diagnostic {
                    range: route.range,
                    severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                    code: Some(lsp_types::NumberOrString::String("E005".to_string())),
                    source: Some("spring-lsp".to_string()),
                    message: format!("路由路径必须以 '/' 开头，当前路径: '{}'", route.path),
                    related_information: None,
                    tags: None,
                    code_description: None,
                    data: None,
                });
            }

            // 检查路径参数格式
            self.validate_path_parameters(&route.path, route.range, &mut diagnostics);
        }

        // 检查是否至少有一个 HTTP 方法
        if route.methods.is_empty() {
            diagnostics.push(lsp_types::Diagnostic {
                range: route.range,
                severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                code: Some(lsp_types::NumberOrString::String("E006".to_string())),
                source: Some("spring-lsp".to_string()),
                message: "路由必须至少指定一个 HTTP 方法".to_string(),
                related_information: None,
                tags: None,
                code_description: None,
                data: None,
            });
        }

        // 检查处理器函数名称是否为空
        if route.handler_name.is_empty() {
            diagnostics.push(lsp_types::Diagnostic {
                range: route.range,
                severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                code: Some(lsp_types::NumberOrString::String("E007".to_string())),
                source: Some("spring-lsp".to_string()),
                message: "路由处理器函数名称不能为空".to_string(),
                related_information: None,
                tags: None,
                code_description: None,
                data: None,
            });
        }

        diagnostics
    }

    /// 验证路径参数格式
    ///
    /// 检查路径中的参数是否符合 {param} 格式
    fn validate_path_parameters(
        &self,
        path: &str,
        range: Range,
        diagnostics: &mut Vec<lsp_types::Diagnostic>,
    ) {
        let mut open_braces = 0;
        let mut param_start = None;

        for (i, ch) in path.chars().enumerate() {
            match ch {
                '{' => {
                    if open_braces > 0 {
                        // 嵌套的大括号
                        diagnostics.push(lsp_types::Diagnostic {
                            range,
                            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                            code: Some(lsp_types::NumberOrString::String("E008".to_string())),
                            source: Some("spring-lsp".to_string()),
                            message: format!("路径参数不能嵌套，位置: {}", i),
                            related_information: None,
                            tags: None,
                            code_description: None,
                            data: None,
                        });
                    }
                    open_braces += 1;
                    param_start = Some(i);
                }
                '}' => {
                    if open_braces == 0 {
                        // 没有匹配的开括号
                        diagnostics.push(lsp_types::Diagnostic {
                            range,
                            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                            code: Some(lsp_types::NumberOrString::String("E009".to_string())),
                            source: Some("spring-lsp".to_string()),
                            message: format!("路径参数缺少开括号 '{{', 位置: {}", i),
                            related_information: None,
                            tags: None,
                            code_description: None,
                            data: None,
                        });
                    } else {
                        open_braces -= 1;

                        // 检查参数名称是否为空
                        if let Some(start) = param_start {
                            let param_name = &path[start + 1..i];
                            if param_name.is_empty() {
                                diagnostics.push(lsp_types::Diagnostic {
                                    range,
                                    severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                                    code: Some(lsp_types::NumberOrString::String(
                                        "E010".to_string(),
                                    )),
                                    source: Some("spring-lsp".to_string()),
                                    message: format!("路径参数名称不能为空，位置: {}", start),
                                    related_information: None,
                                    tags: None,
                                    code_description: None,
                                    data: None,
                                });
                            } else if !param_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                                // 检查参数名称是否只包含字母、数字和下划线
                                diagnostics.push(lsp_types::Diagnostic {
                                    range,
                                    severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                                    code: Some(lsp_types::NumberOrString::String(
                                        "E011".to_string(),
                                    )),
                                    source: Some("spring-lsp".to_string()),
                                    message: format!(
                                        "路径参数名称只能包含字母、数字和下划线，当前参数: '{}'",
                                        param_name
                                    ),
                                    related_information: None,
                                    tags: None,
                                    code_description: None,
                                    data: None,
                                });
                            }
                        }
                        param_start = None;
                    }
                }
                _ => {}
            }
        }

        // 检查是否有未闭合的括号
        if open_braces > 0 {
            diagnostics.push(lsp_types::Diagnostic {
                range,
                severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                code: Some(lsp_types::NumberOrString::String("E012".to_string())),
                source: Some("spring-lsp".to_string()),
                message: "路径参数缺少闭括号 '}'".to_string(),
                related_information: None,
                tags: None,
                code_description: None,
                data: None,
            });
        }
    }

    /// 验证任务调度宏
    ///
    /// 检查 cron 表达式、延迟和频率值是否有效
    fn validate_job_macro(&self, job: &JobMacro) -> Vec<lsp_types::Diagnostic> {
        let mut diagnostics = Vec::new();

        match job {
            JobMacro::Cron { expression, range } => {
                // 检查 cron 表达式是否为空
                if expression.is_empty() {
                    diagnostics.push(lsp_types::Diagnostic {
                        range: *range,
                        severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                        code: Some(lsp_types::NumberOrString::String("E013".to_string())),
                        source: Some("spring-lsp".to_string()),
                        message: "Cron 表达式不能为空".to_string(),
                        related_information: None,
                        tags: None,
                        code_description: None,
                        data: None,
                    });
                } else {
                    // 验证 cron 表达式格式（基本验证）
                    self.validate_cron_expression(expression, *range, &mut diagnostics);
                }
            }
            JobMacro::FixDelay { seconds, range } => {
                // 检查延迟秒数是否为 0
                if *seconds == 0 {
                    diagnostics.push(lsp_types::Diagnostic {
                        range: *range,
                        severity: Some(lsp_types::DiagnosticSeverity::WARNING),
                        code: Some(lsp_types::NumberOrString::String("W001".to_string())),
                        source: Some("spring-lsp".to_string()),
                        message: "延迟秒数为 0 可能不是预期的行为".to_string(),
                        related_information: None,
                        tags: None,
                        code_description: None,
                        data: None,
                    });
                }
            }
            JobMacro::FixRate { seconds, range } => {
                // 检查频率秒数是否为 0
                if *seconds == 0 {
                    diagnostics.push(lsp_types::Diagnostic {
                        range: *range,
                        severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                        code: Some(lsp_types::NumberOrString::String("E014".to_string())),
                        source: Some("spring-lsp".to_string()),
                        message: "频率秒数不能为 0".to_string(),
                        related_information: None,
                        tags: None,
                        code_description: None,
                        data: None,
                    });
                }
            }
        }

        diagnostics
    }

    /// 验证 cron 表达式格式
    ///
    /// 基本验证 cron 表达式是否符合 "秒 分 时 日 月 星期" 格式
    fn validate_cron_expression(
        &self,
        expression: &str,
        range: Range,
        diagnostics: &mut Vec<lsp_types::Diagnostic>,
    ) {
        let parts: Vec<&str> = expression.split_whitespace().collect();

        // Cron 表达式应该有 6 个部分：秒 分 时 日 月 星期
        if parts.len() != 6 {
            diagnostics.push(lsp_types::Diagnostic {
                range,
                severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                code: Some(lsp_types::NumberOrString::String("E015".to_string())),
                source: Some("spring-lsp".to_string()),
                message: format!(
                    "Cron 表达式应该包含 6 个部分（秒 分 时 日 月 星期），当前有 {} 个部分",
                    parts.len()
                ),
                related_information: None,
                tags: None,
                code_description: None,
                data: None,
            });
        }
    }
}

impl Default for MacroAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
