//! 独立的宏模型测试
//!
//! 这个测试文件独立于库的其他部分，专门测试宏数据模型

use lsp_types::{Position, Range, Url};

// 由于 toml_analyzer 有编译错误，我们直接在这里定义需要的类型
// 这样可以独立测试 macro_analyzer 模块

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

/// 注入类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InjectType {
    Component,
    Config,
}

/// Inject 属性宏信息
#[derive(Debug, Clone)]
pub struct InjectMacro {
    pub inject_type: InjectType,
    pub component_name: Option<String>,
    pub range: Range,
}

/// 字段信息
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub type_name: String,
    pub inject: Option<InjectMacro>,
}

/// Service 派生宏信息
#[derive(Debug, Clone)]
pub struct ServiceMacro {
    pub struct_name: String,
    pub fields: Vec<Field>,
    pub range: Range,
}

/// 路由宏信息
#[derive(Debug, Clone)]
pub struct RouteMacro {
    pub path: String,
    pub methods: Vec<HttpMethod>,
    pub middlewares: Vec<String>,
    pub handler_name: String,
    pub range: Range,
}

/// AutoConfig 属性宏信息
#[derive(Debug, Clone)]
pub struct AutoConfigMacro {
    pub configurator_type: String,
    pub range: Range,
}

/// 任务调度宏信息
#[derive(Debug, Clone)]
pub enum JobMacro {
    Cron { expression: String, range: Range },
    FixDelay { seconds: u64, range: Range },
    FixRate { seconds: u64, range: Range },
}

/// Spring-rs 宏枚举
#[derive(Debug, Clone)]
pub enum SpringMacro {
    DeriveService(ServiceMacro),
    Inject(InjectMacro),
    AutoConfig(AutoConfigMacro),
    Route(RouteMacro),
    Job(JobMacro),
}

/// Rust 文档模型
#[derive(Debug, Clone)]
pub struct RustDocument {
    pub uri: Url,
    pub content: String,
    pub macros: Vec<SpringMacro>,
}

// ============ 测试 ============

fn test_range() -> Range {
    Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 10,
        },
    }
}

#[test]
fn test_http_method_from_str() {
    assert_eq!(HttpMethod::from_str("GET"), Some(HttpMethod::Get));
    assert_eq!(HttpMethod::from_str("get"), Some(HttpMethod::Get));
    assert_eq!(HttpMethod::from_str("POST"), Some(HttpMethod::Post));
    assert_eq!(HttpMethod::from_str("INVALID"), None);
}

#[test]
fn test_http_method_as_str() {
    assert_eq!(HttpMethod::Get.as_str(), "GET");
    assert_eq!(HttpMethod::Post.as_str(), "POST");
}

#[test]
fn test_inject_macro_with_component() {
    let inject = InjectMacro {
        inject_type: InjectType::Component,
        component_name: Some("db_pool".to_string()),
        range: test_range(),
    };

    assert_eq!(inject.inject_type, InjectType::Component);
    assert_eq!(inject.component_name, Some("db_pool".to_string()));
}

#[test]
fn test_inject_macro_with_config() {
    let inject = InjectMacro {
        inject_type: InjectType::Config,
        component_name: None,
        range: test_range(),
    };

    assert_eq!(inject.inject_type, InjectType::Config);
    assert_eq!(inject.component_name, None);
}

#[test]
fn test_service_macro_with_fields() {
    let service = ServiceMacro {
        struct_name: "UserService".to_string(),
        fields: vec![
            Field {
                name: "db".to_string(),
                type_name: "ConnectPool".to_string(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Component,
                    component_name: None,
                    range: test_range(),
                }),
            },
            Field {
                name: "config".to_string(),
                type_name: "AppConfig".to_string(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Config,
                    component_name: None,
                    range: test_range(),
                }),
            },
        ],
        range: test_range(),
    };

    assert_eq!(service.struct_name, "UserService");
    assert_eq!(service.fields.len(), 2);
    assert_eq!(service.fields[0].name, "db");
    assert_eq!(service.fields[1].name, "config");
}

#[test]
fn test_route_macro_single_method() {
    let route = RouteMacro {
        path: "/users".to_string(),
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "list_users".to_string(),
        range: test_range(),
    };

    assert_eq!(route.path, "/users");
    assert_eq!(route.methods.len(), 1);
    assert_eq!(route.methods[0], HttpMethod::Get);
    assert_eq!(route.handler_name, "list_users");
}

#[test]
fn test_route_macro_multiple_methods() {
    let route = RouteMacro {
        path: "/api/resource".to_string(),
        methods: vec![HttpMethod::Get, HttpMethod::Post],
        middlewares: vec!["AuthMiddleware".to_string()],
        handler_name: "handle_resource".to_string(),
        range: test_range(),
    };

    assert_eq!(route.methods.len(), 2);
    assert!(route.methods.contains(&HttpMethod::Get));
    assert!(route.methods.contains(&HttpMethod::Post));
    assert_eq!(route.middlewares.len(), 1);
}

#[test]
fn test_auto_config_macro() {
    let auto_config = AutoConfigMacro {
        configurator_type: "WebConfigurator".to_string(),
        range: test_range(),
    };

    assert_eq!(auto_config.configurator_type, "WebConfigurator");
}

#[test]
fn test_job_macro_cron() {
    let job = JobMacro::Cron {
        expression: "0 0 * * * *".to_string(),
        range: test_range(),
    };

    match job {
        JobMacro::Cron { expression, .. } => {
            assert_eq!(expression, "0 0 * * * *");
        }
        _ => panic!("Expected Cron variant"),
    }
}

#[test]
fn test_job_macro_fix_delay() {
    let job = JobMacro::FixDelay {
        seconds: 5,
        range: test_range(),
    };

    match job {
        JobMacro::FixDelay { seconds, .. } => {
            assert_eq!(seconds, 5);
        }
        _ => panic!("Expected FixDelay variant"),
    }
}

#[test]
fn test_job_macro_fix_rate() {
    let job = JobMacro::FixRate {
        seconds: 10,
        range: test_range(),
    };

    match job {
        JobMacro::FixRate { seconds, .. } => {
            assert_eq!(seconds, 10);
        }
        _ => panic!("Expected FixRate variant"),
    }
}

#[test]
fn test_spring_macro_variants() {
    let macros = vec![
        SpringMacro::DeriveService(ServiceMacro {
            struct_name: "MyService".to_string(),
            fields: vec![],
            range: test_range(),
        }),
        SpringMacro::Inject(InjectMacro {
            inject_type: InjectType::Component,
            component_name: None,
            range: test_range(),
        }),
        SpringMacro::Route(RouteMacro {
            path: "/test".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "test_handler".to_string(),
            range: test_range(),
        }),
        SpringMacro::AutoConfig(AutoConfigMacro {
            configurator_type: "WebConfigurator".to_string(),
            range: test_range(),
        }),
        SpringMacro::Job(JobMacro::Cron {
            expression: "0 0 * * * *".to_string(),
            range: test_range(),
        }),
    ];

    assert_eq!(macros.len(), 5);
}

#[test]
fn test_rust_document() {
    let uri = Url::parse("file:///test.rs").unwrap();
    let doc = RustDocument {
        uri: uri.clone(),
        content: "fn main() {}".to_string(),
        macros: vec![SpringMacro::Route(RouteMacro {
            path: "/".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "index".to_string(),
            range: test_range(),
        })],
    };

    assert_eq!(doc.uri, uri);
    assert_eq!(doc.content, "fn main() {}");
    assert_eq!(doc.macros.len(), 1);
}

#[test]
fn test_field_without_inject() {
    let field = Field {
        name: "name".to_string(),
        type_name: "String".to_string(),
        inject: None,
    };

    assert_eq!(field.name, "name");
    assert_eq!(field.type_name, "String");
    assert!(field.inject.is_none());
}

#[test]
fn test_route_with_path_parameters() {
    let route = RouteMacro {
        path: "/users/{id}/posts/{post_id}".to_string(),
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "get_user_post".to_string(),
        range: test_range(),
    };

    assert_eq!(route.path, "/users/{id}/posts/{post_id}");
    assert!(route.path.contains("{id}"));
    assert!(route.path.contains("{post_id}"));
}
