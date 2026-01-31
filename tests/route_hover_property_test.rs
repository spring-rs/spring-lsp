//! 路由悬停提示属性测试
//!
//! 使用 proptest 验证路由宏悬停提示功能的正确性

use proptest::prelude::*;
use spring_lsp::macro_analyzer::{HttpMethod, MacroAnalyzer, RouteMacro, SpringMacro};
use lsp_types::{Position, Range, Url};

// 配置 proptest 运行至少 100 次迭代
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        .. ProptestConfig::default()
    })]
}

// ============================================================================
// 测试数据生成器
// ============================================================================

/// 生成有效的 Rust 标识符
fn valid_identifier() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_]{0,30}"
}

/// 生成有效的路由路径
fn valid_route_path() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("/".to_string()),
        Just("/users".to_string()),
        Just("/api/v1/users".to_string()),
        Just("/posts/{id}".to_string()),
        Just("/users/{user_id}/posts/{post_id}".to_string()),
        "[a-z/]{1,30}".prop_map(|s| format!("/{}", s.replace("//", "/"))),
        (valid_identifier(), valid_identifier())
            .prop_map(|(seg1, seg2)| format!("/{}/{}", seg1, seg2)),
    ]
}

/// 生成 HTTP 方法列表
fn http_methods() -> impl Strategy<Value = Vec<HttpMethod>> {
    prop::collection::vec(
        prop_oneof![
            Just(HttpMethod::Get),
            Just(HttpMethod::Post),
            Just(HttpMethod::Put),
            Just(HttpMethod::Delete),
            Just(HttpMethod::Patch),
            Just(HttpMethod::Head),
            Just(HttpMethod::Options),
            Just(HttpMethod::Connect),
            Just(HttpMethod::Trace),
        ],
        1..5
    )
}

/// 生成中间件列表
fn middlewares() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(
        valid_identifier(),
        0..3
    )
}

/// 生成 RouteMacro
fn route_macro() -> impl Strategy<Value = RouteMacro> {
    (
        valid_route_path(),
        http_methods(),
        middlewares(),
        valid_identifier(),
    )
        .prop_map(|(path, methods, middlewares, handler_name)| {
            RouteMacro {
                path,
                methods,
                middlewares,
                handler_name,
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 0 },
                },
            }
        })
}

/// 生成测试用的 URI
fn test_uri() -> impl Strategy<Value = Url> {
    valid_identifier()
        .prop_map(|name| {
            Url::parse(&format!("file:///test/{}.rs", name))
                .expect("Failed to create test URI")
        })
}

// ============================================================================
// 属性测试
// ============================================================================

// Feature: spring-lsp, Property 62: 路由宏悬停信息
//
// **Validates: Requirements 15.2**
//
// *For any* 路由宏，悬停时应该显示完整的路由路径和 HTTP 方法列表。
//
// 这个属性测试验证：
// 1. 对于任何路由宏，hover_macro 应该返回非空的悬停文本
// 2. 悬停文本应该包含路由路径
// 3. 悬停文本应该包含所有 HTTP 方法
// 4. 悬停文本应该包含处理器函数名称
proptest! {
    #[test]
    fn prop_route_hover_contains_path_and_methods(
        route in route_macro()
    ) {
        let analyzer = MacroAnalyzer::new();
        let spring_macro = SpringMacro::Route(route.clone());
        
        // 获取悬停文本
        let hover_text = analyzer.hover_macro(&spring_macro);
        
        // 悬停文本不应为空
        prop_assert!(!hover_text.is_empty(),
            "Hover text should not be empty");
        
        // 悬停文本应该包含路由路径
        prop_assert!(hover_text.contains(&route.path),
            "Hover text should contain route path '{}', but got:\n{}",
            route.path, hover_text);
        
        // 悬停文本应该包含所有 HTTP 方法
        for method in &route.methods {
            let method_str = method.as_str();
            prop_assert!(hover_text.contains(method_str),
                "Hover text should contain HTTP method '{}', but got:\n{}",
                method_str, hover_text);
        }
        
        // 悬停文本应该包含处理器函数名称
        prop_assert!(hover_text.contains(&route.handler_name),
            "Hover text should contain handler name '{}', but got:\n{}",
            route.handler_name, hover_text);
    }
}

// Feature: spring-lsp, Property 62: 路由宏悬停信息（单个方法）
//
// **Validates: Requirements 15.2**
//
// 验证单个 HTTP 方法的路由悬停信息
proptest! {
    #[test]
    fn prop_route_hover_single_method(
        path in valid_route_path(),
        method in prop_oneof![
            Just(HttpMethod::Get),
            Just(HttpMethod::Post),
            Just(HttpMethod::Put),
            Just(HttpMethod::Delete),
        ],
        handler_name in valid_identifier()
    ) {
        let analyzer = MacroAnalyzer::new();
        let route = RouteMacro {
            path: path.clone(),
            methods: vec![method.clone()],
            middlewares: vec![],
            handler_name: handler_name.clone(),
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 0 },
            },
        };
        let spring_macro = SpringMacro::Route(route);
        
        let hover_text = analyzer.hover_macro(&spring_macro);
        
        // 验证包含路径和方法
        prop_assert!(hover_text.contains(&path),
            "Hover text should contain path");
        prop_assert!(hover_text.contains(method.as_str()),
            "Hover text should contain method");
        prop_assert!(hover_text.contains(&handler_name),
            "Hover text should contain handler name");
    }
}

// Feature: spring-lsp, Property 62: 路由宏悬停信息（多个方法）
//
// **Validates: Requirements 15.2**
//
// 验证多个 HTTP 方法的路由悬停信息都能正确显示
proptest! {
    #[test]
    fn prop_route_hover_multiple_methods(
        path in valid_route_path(),
        handler_name in valid_identifier()
    ) {
        let analyzer = MacroAnalyzer::new();
        let methods = vec![
            HttpMethod::Get,
            HttpMethod::Post,
            HttpMethod::Put,
        ];
        let route = RouteMacro {
            path: path.clone(),
            methods: methods.clone(),
            middlewares: vec![],
            handler_name: handler_name.clone(),
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 0 },
            },
        };
        let spring_macro = SpringMacro::Route(route);
        
        let hover_text = analyzer.hover_macro(&spring_macro);
        
        // 验证包含所有方法
        for method in &methods {
            prop_assert!(hover_text.contains(method.as_str()),
                "Hover text should contain method '{}'", method.as_str());
        }
    }
}

// Feature: spring-lsp, Property 62: 路由宏悬停信息（带中间件）
//
// **Validates: Requirements 15.2**
//
// 验证带有中间件的路由悬停信息
proptest! {
    #[test]
    fn prop_route_hover_with_middlewares(
        path in valid_route_path(),
        method in prop_oneof![
            Just(HttpMethod::Get),
            Just(HttpMethod::Post),
        ],
        handler_name in valid_identifier(),
        middleware_names in prop::collection::vec(valid_identifier(), 1..3)
    ) {
        let analyzer = MacroAnalyzer::new();
        let route = RouteMacro {
            path: path.clone(),
            methods: vec![method],
            middlewares: middleware_names.clone(),
            handler_name: handler_name.clone(),
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 0 },
            },
        };
        let spring_macro = SpringMacro::Route(route);
        
        let hover_text = analyzer.hover_macro(&spring_macro);
        
        // 如果有中间件，悬停文本应该包含中间件信息
        if !middleware_names.is_empty() {
            for middleware in &middleware_names {
                prop_assert!(hover_text.contains(middleware),
                    "Hover text should contain middleware '{}'", middleware);
            }
        }
    }
}

// Feature: spring-lsp, Property 62: 路由宏悬停信息（路径参数）
//
// **Validates: Requirements 15.2**
//
// 验证带有路径参数的路由悬停信息
proptest! {
    #[test]
    fn prop_route_hover_with_path_params(
        param_name in valid_identifier(),
        handler_name in valid_identifier()
    ) {
        let analyzer = MacroAnalyzer::new();
        let path = format!("/users/{{{}}}", param_name);
        let route = RouteMacro {
            path: path.clone(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: handler_name.clone(),
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 0 },
            },
        };
        let spring_macro = SpringMacro::Route(route);
        
        let hover_text = analyzer.hover_macro(&spring_macro);
        
        // 悬停文本应该包含完整的路径（包括参数）
        prop_assert!(hover_text.contains(&path),
            "Hover text should contain path with parameters");
    }
}

// Feature: spring-lsp, Property 62: 路由宏悬停信息（格式化）
//
// **Validates: Requirements 15.2**
//
// 验证悬停文本的格式化（应该是 Markdown 格式）
proptest! {
    #[test]
    fn prop_route_hover_markdown_format(
        route in route_macro()
    ) {
        let analyzer = MacroAnalyzer::new();
        let spring_macro = SpringMacro::Route(route);
        
        let hover_text = analyzer.hover_macro(&spring_macro);
        
        // 悬停文本应该包含 Markdown 标题
        prop_assert!(hover_text.contains("# ") || hover_text.contains("## "),
            "Hover text should contain Markdown headers");
        
        // 悬停文本应该包含代码块标记（用于展开后的代码）
        prop_assert!(hover_text.contains("```"),
            "Hover text should contain code blocks");
    }
}

// Feature: spring-lsp, Property 62: 路由宏悬停信息（一致性）
//
// **Validates: Requirements 15.2**
//
// 验证相同的路由宏多次调用 hover_macro 应该返回相同的结果
proptest! {
    #[test]
    fn prop_route_hover_consistency(
        route in route_macro()
    ) {
        let analyzer = MacroAnalyzer::new();
        let spring_macro = SpringMacro::Route(route);
        
        // 多次调用 hover_macro
        let hover_text1 = analyzer.hover_macro(&spring_macro);
        let hover_text2 = analyzer.hover_macro(&spring_macro);
        let hover_text3 = analyzer.hover_macro(&spring_macro);
        
        // 结果应该相同
        prop_assert_eq!(&hover_text1, &hover_text2,
            "Multiple calls to hover_macro should return the same result");
        prop_assert_eq!(&hover_text2, &hover_text3,
            "Multiple calls to hover_macro should return the same result");
    }
}

// Feature: spring-lsp, Property 62: 路由宏悬停信息（非空路径）
//
// **Validates: Requirements 15.2**
//
// 验证即使是最简单的路由也应该有有意义的悬停信息
proptest! {
    #[test]
    fn prop_route_hover_minimal_route(
        handler_name in valid_identifier()
    ) {
        let analyzer = MacroAnalyzer::new();
        let route = RouteMacro {
            path: "/".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: handler_name.clone(),
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 0 },
            },
        };
        let spring_macro = SpringMacro::Route(route);
        
        let hover_text = analyzer.hover_macro(&spring_macro);
        
        // 即使是最简单的路由，悬停文本也应该有实质内容
        prop_assert!(hover_text.len() > 50,
            "Hover text should have substantial content even for minimal routes");
        
        // 应该包含基本信息
        prop_assert!(hover_text.contains("/"),
            "Hover text should contain the root path");
        prop_assert!(hover_text.contains("GET"),
            "Hover text should contain the HTTP method");
        prop_assert!(hover_text.contains(&handler_name),
            "Hover text should contain the handler name");
    }
}

// Feature: spring-lsp, Property 62: 路由宏悬停信息（所有 HTTP 方法）
//
// **Validates: Requirements 15.2**
//
// 验证所有支持的 HTTP 方法都能正确显示在悬停信息中
proptest! {
    #[test]
    fn prop_route_hover_all_http_methods(
        path in valid_route_path(),
        handler_name in valid_identifier()
    ) {
        let analyzer = MacroAnalyzer::new();
        
        // 测试所有 HTTP 方法
        let all_methods = vec![
            HttpMethod::Get,
            HttpMethod::Post,
            HttpMethod::Put,
            HttpMethod::Delete,
            HttpMethod::Patch,
            HttpMethod::Head,
            HttpMethod::Options,
            HttpMethod::Connect,
            HttpMethod::Trace,
        ];
        
        for method in &all_methods {
            let route = RouteMacro {
                path: path.clone(),
                methods: vec![method.clone()],
                middlewares: vec![],
                handler_name: handler_name.clone(),
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 0 },
                },
            };
            let spring_macro = SpringMacro::Route(route);
            
            let hover_text = analyzer.hover_macro(&spring_macro);
            
            // 悬停文本应该包含该方法
            prop_assert!(hover_text.contains(method.as_str()),
                "Hover text should contain HTTP method '{}'", method.as_str());
        }
    }
}
