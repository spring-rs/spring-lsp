//! 补全引擎单元测试

use super::*;
use crate::macro_analyzer::{
    AutoConfigMacro, HttpMethod, InjectMacro, InjectType, JobMacro, RouteMacro, ServiceMacro,
    SpringMacro,
};
use lsp_types::{Position, Range, Url};

/// 创建测试用的 Range
fn test_range() -> Range {
    Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    }
}

/// 创建测试用的 URL
fn test_url() -> Url {
    Url::parse("file:///test.rs").unwrap()
}

#[test]
fn test_complete_service_macro() {
    let engine = CompletionEngine::new();
    let service_macro = ServiceMacro {
        struct_name: "TestService".to_string(),
        fields: vec![],
        range: test_range(),
    };

    let completions = engine.complete_macro(&SpringMacro::DeriveService(service_macro), None);

    // 应该提供 3 个补全项：inject(component), inject(component = "name"), inject(config)
    assert_eq!(completions.len(), 3);

    // 检查第一个补全项：inject(component)
    assert_eq!(completions[0].label, "inject(component)");
    assert_eq!(completions[0].kind, Some(CompletionItemKind::PROPERTY));
    assert_eq!(completions[0].detail, Some("注入组件".to_string()));
    assert!(completions[0].documentation.is_some());
    assert_eq!(
        completions[0].insert_text,
        Some("inject(component)".to_string())
    );

    // 检查第二个补全项：inject(component = "name")
    assert_eq!(completions[1].label, "inject(component = \"name\")");
    assert_eq!(completions[1].kind, Some(CompletionItemKind::PROPERTY));
    assert_eq!(
        completions[1].detail,
        Some("注入指定名称的组件".to_string())
    );
    assert!(completions[1].documentation.is_some());
    assert_eq!(
        completions[1].insert_text,
        Some("inject(component = \"$1\")".to_string())
    );
    assert_eq!(
        completions[1].insert_text_format,
        Some(lsp_types::InsertTextFormat::SNIPPET)
    );

    // 检查第三个补全项：inject(config)
    assert_eq!(completions[2].label, "inject(config)");
    assert_eq!(completions[2].kind, Some(CompletionItemKind::PROPERTY));
    assert_eq!(completions[2].detail, Some("注入配置".to_string()));
    assert!(completions[2].documentation.is_some());
    assert_eq!(
        completions[2].insert_text,
        Some("inject(config)".to_string())
    );
}

#[test]
fn test_complete_inject_macro() {
    let engine = CompletionEngine::new();
    let inject_macro = InjectMacro {
        inject_type: InjectType::Component,
        component_name: None,
        range: test_range(),
    };

    let completions = engine.complete_macro(&SpringMacro::Inject(inject_macro), None);

    // 应该提供 2 个补全项：component, config
    assert_eq!(completions.len(), 2);

    // 检查第一个补全项：component
    assert_eq!(completions[0].label, "component");
    assert_eq!(completions[0].kind, Some(CompletionItemKind::KEYWORD));
    assert_eq!(completions[0].detail, Some("注入组件".to_string()));
    assert!(completions[0].documentation.is_some());
    assert_eq!(completions[0].insert_text, Some("component".to_string()));

    // 检查第二个补全项：config
    assert_eq!(completions[1].label, "config");
    assert_eq!(completions[1].kind, Some(CompletionItemKind::KEYWORD));
    assert_eq!(completions[1].detail, Some("注入配置".to_string()));
    assert!(completions[1].documentation.is_some());
    assert_eq!(completions[1].insert_text, Some("config".to_string()));
}

#[test]
fn test_complete_auto_config_macro() {
    let engine = CompletionEngine::new();
    let auto_config_macro = AutoConfigMacro {
        configurator_type: "".to_string(),
        range: test_range(),
    };

    let completions = engine.complete_macro(&SpringMacro::AutoConfig(auto_config_macro), None);

    // 应该提供 3 个配置器类型补全
    assert_eq!(completions.len(), 3);

    // 检查补全项
    assert_eq!(completions[0].label, "WebConfigurator");
    assert_eq!(completions[0].kind, Some(CompletionItemKind::CLASS));
    assert_eq!(completions[0].detail, Some("Web 路由配置器".to_string()));
    assert!(completions[0].documentation.is_some());

    assert_eq!(completions[1].label, "JobConfigurator");
    assert_eq!(completions[1].kind, Some(CompletionItemKind::CLASS));
    assert_eq!(completions[1].detail, Some("任务调度配置器".to_string()));
    assert!(completions[1].documentation.is_some());

    assert_eq!(completions[2].label, "StreamConfigurator");
    assert_eq!(completions[2].kind, Some(CompletionItemKind::CLASS));
    assert_eq!(completions[2].detail, Some("流处理配置器".to_string()));
    assert!(completions[2].documentation.is_some());
}

#[test]
fn test_complete_route_macro() {
    let engine = CompletionEngine::new();
    let route_macro = RouteMacro {
        path: "/test".to_string(),
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "test_handler".to_string(),
        range: test_range(),
    };

    let completions = engine.complete_macro(&SpringMacro::Route(route_macro), None);

    // 应该提供 HTTP 方法和路径参数补全
    assert!(completions.len() >= 7); // 至少 7 个 HTTP 方法

    // 检查 HTTP 方法补全
    let get_completion = completions.iter().find(|c| c.label == "GET");
    assert!(get_completion.is_some());
    let get_completion = get_completion.unwrap();
    assert_eq!(get_completion.kind, Some(CompletionItemKind::CONSTANT));
    assert_eq!(get_completion.detail, Some("获取资源".to_string()));
    assert!(get_completion.documentation.is_some());

    let post_completion = completions.iter().find(|c| c.label == "POST");
    assert!(post_completion.is_some());
    let post_completion = post_completion.unwrap();
    assert_eq!(post_completion.kind, Some(CompletionItemKind::CONSTANT));
    assert_eq!(post_completion.detail, Some("创建资源".to_string()));

    // 检查路径参数补全
    let path_param_completion = completions.iter().find(|c| c.label == "{id}");
    assert!(path_param_completion.is_some());
    let path_param_completion = path_param_completion.unwrap();
    assert_eq!(path_param_completion.kind, Some(CompletionItemKind::SNIPPET));
    assert_eq!(path_param_completion.detail, Some("路径参数".to_string()));
    assert_eq!(
        path_param_completion.insert_text,
        Some("{${1:id}}".to_string())
    );
    assert_eq!(
        path_param_completion.insert_text_format,
        Some(lsp_types::InsertTextFormat::SNIPPET)
    );
}

#[test]
fn test_complete_job_macro_cron() {
    let engine = CompletionEngine::new();
    let job_macro = JobMacro::Cron {
        expression: "".to_string(),
        range: test_range(),
    };

    let completions = engine.complete_macro(&SpringMacro::Job(job_macro), None);

    // 应该提供 cron 表达式和延迟/频率值补全
    assert!(completions.len() >= 6);

    // 检查 cron 表达式补全
    let hourly_cron = completions.iter().find(|c| c.label == "0 0 * * * *");
    assert!(hourly_cron.is_some());
    let hourly_cron = hourly_cron.unwrap();
    assert_eq!(hourly_cron.kind, Some(CompletionItemKind::SNIPPET));
    assert_eq!(hourly_cron.detail, Some("每小时执行".to_string()));
    assert!(hourly_cron.documentation.is_some());
    assert_eq!(
        hourly_cron.insert_text,
        Some("\"0 0 * * * *\"".to_string())
    );

    let daily_cron = completions.iter().find(|c| c.label == "0 0 0 * * *");
    assert!(daily_cron.is_some());
    let daily_cron = daily_cron.unwrap();
    assert_eq!(daily_cron.detail, Some("每天午夜执行".to_string()));

    // 检查延迟/频率值补全
    let delay_5 = completions.iter().find(|c| c.label == "5");
    assert!(delay_5.is_some());
    let delay_5 = delay_5.unwrap();
    assert_eq!(delay_5.kind, Some(CompletionItemKind::VALUE));
    assert_eq!(delay_5.detail, Some("延迟 5 秒".to_string()));
}

#[test]
fn test_complete_job_macro_fix_delay() {
    let engine = CompletionEngine::new();
    let job_macro = JobMacro::FixDelay {
        seconds: 0,
        range: test_range(),
    };

    let completions = engine.complete_macro(&SpringMacro::Job(job_macro), None);

    // 应该提供延迟值补全
    assert!(completions.len() >= 3);

    // 检查延迟值补全
    let delay_10 = completions.iter().find(|c| c.label == "10");
    assert!(delay_10.is_some());
    let delay_10 = delay_10.unwrap();
    assert_eq!(delay_10.kind, Some(CompletionItemKind::VALUE));
    assert!(delay_10.detail.is_some());
    assert!(delay_10.documentation.is_some());
}

#[test]
fn test_complete_job_macro_fix_rate() {
    let engine = CompletionEngine::new();
    let job_macro = JobMacro::FixRate {
        seconds: 0,
        range: test_range(),
    };

    let completions = engine.complete_macro(&SpringMacro::Job(job_macro), None);

    // 应该提供频率值补全
    assert!(completions.len() >= 3);

    // 检查频率值补全
    let rate_60 = completions.iter().find(|c| c.label == "60");
    assert!(rate_60.is_some());
    let rate_60 = rate_60.unwrap();
    assert_eq!(rate_60.kind, Some(CompletionItemKind::VALUE));
    assert!(rate_60.detail.is_some());
    assert!(rate_60.documentation.is_some());
}

#[test]
fn test_completion_items_have_documentation() {
    let engine = CompletionEngine::new();

    // 测试所有宏类型的补全项都有文档
    let test_cases = vec![
        SpringMacro::DeriveService(ServiceMacro {
            struct_name: "Test".to_string(),
            fields: vec![],
            range: test_range(),
        }),
        SpringMacro::Inject(InjectMacro {
            inject_type: InjectType::Component,
            component_name: None,
            range: test_range(),
        }),
        SpringMacro::AutoConfig(AutoConfigMacro {
            configurator_type: "".to_string(),
            range: test_range(),
        }),
        SpringMacro::Route(RouteMacro {
            path: "/test".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "handler".to_string(),
            range: test_range(),
        }),
        SpringMacro::Job(JobMacro::Cron {
            expression: "".to_string(),
            range: test_range(),
        }),
    ];

    for macro_info in test_cases {
        let completions = engine.complete_macro(&macro_info, None);
        assert!(!completions.is_empty(), "补全列表不应为空");

        for completion in completions {
            assert!(
                completion.documentation.is_some(),
                "补全项 '{}' 应该有文档说明",
                completion.label
            );
            assert!(
                completion.detail.is_some(),
                "补全项 '{}' 应该有详细信息",
                completion.label
            );
            assert!(
                completion.insert_text.is_some(),
                "补全项 '{}' 应该有插入文本",
                completion.label
            );
        }
    }
}

#[test]
fn test_completion_items_have_correct_kind() {
    let engine = CompletionEngine::new();

    // Service 宏的补全项应该是 PROPERTY 类型
    let service_completions = engine.complete_macro(
        &SpringMacro::DeriveService(ServiceMacro {
            struct_name: "Test".to_string(),
            fields: vec![],
            range: test_range(),
        }),
        None,
    );
    for completion in service_completions {
        assert_eq!(
            completion.kind,
            Some(CompletionItemKind::PROPERTY),
            "Service 宏的补全项应该是 PROPERTY 类型"
        );
    }

    // Inject 宏的补全项应该是 KEYWORD 类型
    let inject_completions = engine.complete_macro(
        &SpringMacro::Inject(InjectMacro {
            inject_type: InjectType::Component,
            component_name: None,
            range: test_range(),
        }),
        None,
    );
    for completion in inject_completions {
        assert_eq!(
            completion.kind,
            Some(CompletionItemKind::KEYWORD),
            "Inject 宏的补全项应该是 KEYWORD 类型"
        );
    }

    // AutoConfig 宏的补全项应该是 CLASS 类型
    let auto_config_completions = engine.complete_macro(
        &SpringMacro::AutoConfig(AutoConfigMacro {
            configurator_type: "".to_string(),
            range: test_range(),
        }),
        None,
    );
    for completion in auto_config_completions {
        assert_eq!(
            completion.kind,
            Some(CompletionItemKind::CLASS),
            "AutoConfig 宏的补全项应该是 CLASS 类型"
        );
    }
}
