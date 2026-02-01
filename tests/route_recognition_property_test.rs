//! 路由识别属性测试
//!
//! 使用 proptest 验证路由识别功能在随机生成的输入下的正确性

use lsp_types::{Position, Range, Url};
use proptest::prelude::*;
use spring_lsp::macro_analyzer::{HttpMethod, RouteMacro, RustDocument, SpringMacro};
use spring_lsp::route::RouteNavigator;

// ============================================================================
// 测试策略：生成有效的路由数据
// ============================================================================

/// 生成有效的路由路径
fn valid_route_path() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "/users".to_string(),
        "/users/{id}".to_string(),
        "/api/v1/users".to_string(),
        "/api/v1/users/{id}".to_string(),
        "/posts/{post_id}/comments".to_string(),
        "/posts/{post_id}/comments/{id}".to_string(),
        "/api/v2/users/{user_id}/posts/{post_id}".to_string(),
        "/".to_string(),
        "/health".to_string(),
        "/api/status".to_string(),
    ])
}

/// 生成有效的路由路径（包含参数）
fn route_path_with_params() -> impl Strategy<Value = (String, Vec<String>)> {
    prop::sample::select(vec![
        ("/users/{id}".to_string(), vec!["id".to_string()]),
        (
            "/users/{user_id}/posts/{post_id}".to_string(),
            vec!["user_id".to_string(), "post_id".to_string()],
        ),
        ("/api/v1/users/{id}".to_string(), vec!["id".to_string()]),
        (
            "/posts/{post_id}/comments/{comment_id}".to_string(),
            vec!["post_id".to_string(), "comment_id".to_string()],
        ),
        (
            "/api/{version}/users/{id}".to_string(),
            vec!["version".to_string(), "id".to_string()],
        ),
    ])
}

/// 生成有效的处理器函数名
fn valid_handler_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,30}".prop_map(|s| s)
}

/// 生成单个 HTTP 方法
fn single_http_method() -> impl Strategy<Value = HttpMethod> {
    prop::sample::select(vec![
        HttpMethod::Get,
        HttpMethod::Post,
        HttpMethod::Put,
        HttpMethod::Delete,
        HttpMethod::Patch,
        HttpMethod::Head,
        HttpMethod::Options,
    ])
}

/// 生成多个 HTTP 方法
fn multiple_http_methods() -> impl Strategy<Value = Vec<HttpMethod>> {
    prop::collection::vec(single_http_method(), 1..4).prop_map(|methods| {
        // 去重
        let mut unique_methods = Vec::new();
        for method in methods {
            if !unique_methods.contains(&method) {
                unique_methods.push(method);
            }
        }
        unique_methods
    })
}

/// 生成有效的位置范围
fn valid_range() -> impl Strategy<Value = Range> {
    (0u32..1000, 0u32..100, 0u32..1000, 0u32..100).prop_map(
        |(start_line, start_char, end_line, end_char)| {
            let (start_line, end_line, start_char, end_char) = if start_line < end_line {
                // 不同行，字符位置无所谓
                (start_line, end_line, start_char, end_char)
            } else if start_line > end_line {
                // 交换行
                (end_line, start_line, end_char, start_char)
            } else {
                // 同一行，确保起始字符 <= 结束字符
                if start_char <= end_char {
                    (start_line, end_line, start_char, end_char)
                } else {
                    (start_line, end_line, end_char, start_char)
                }
            };

            Range {
                start: Position {
                    line: start_line,
                    character: start_char,
                },
                end: Position {
                    line: end_line,
                    character: end_char,
                },
            }
        },
    )
}

/// 生成有效的文件 URI
fn valid_file_uri() -> impl Strategy<Value = Url> {
    "[a-z][a-z0-9_]{0,20}\\.rs"
        .prop_map(|filename| Url::parse(&format!("file:///{}", filename)).unwrap())
}

/// 生成单个路由宏
fn single_route_macro() -> impl Strategy<Value = RouteMacro> {
    (
        valid_route_path(),
        multiple_http_methods(),
        valid_handler_name(),
        valid_range(),
    )
        .prop_map(|(path, methods, handler_name, range)| RouteMacro {
            path,
            methods,
            middlewares: vec![],
            handler_name,
            range,
        })
}

/// 生成包含路由宏的 Rust 文档
fn rust_document_with_routes() -> impl Strategy<Value = RustDocument> {
    (
        valid_file_uri(),
        prop::collection::vec(single_route_macro(), 1..10),
    )
        .prop_map(|(uri, route_macros)| {
            let macros = route_macros.into_iter().map(SpringMacro::Route).collect();

            RustDocument {
                uri,
                content: String::new(),
                macros,
            }
        })
}

/// 生成多个 Rust 文档
fn multiple_rust_documents() -> impl Strategy<Value = Vec<RustDocument>> {
    prop::collection::vec(rust_document_with_routes(), 1..5)
}

// ============================================================================
// Property 36: 路由完整识别
// **Validates: Requirements 8.1**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 36: 路由完整识别
    ///
    /// For any 项目中的 Rust 文件，路由导航器应该识别所有路由宏标注的函数。
    #[test]
    fn prop_route_complete_recognition(
        documents in multiple_rust_documents()
    ) {
        let mut navigator = RouteNavigator::new();

        // 计算预期的路由数量（考虑多方法路由会展开）
        let expected_route_count: usize = documents
            .iter()
            .flat_map(|doc| &doc.macros)
            .filter_map(|m| {
                if let SpringMacro::Route(route) = m {
                    Some(route.methods.len())
                } else {
                    None
                }
            })
            .sum();

        // 构建索引
        navigator.build_index(&documents);

        // 属性：识别的路由数量应该等于预期数量
        prop_assert_eq!(
            navigator.index.routes.len(),
            expected_route_count,
            "应该识别所有路由宏标注的函数"
        );

        // 属性：每个路由都应该有有效的路径
        for route in &navigator.index.routes {
            prop_assert!(!route.path.is_empty(), "路由路径不应为空");
        }

        // 属性：每个路由都应该有至少一个 HTTP 方法
        for route in &navigator.index.routes {
            prop_assert!(!route.methods.is_empty(), "路由应该至少有一个 HTTP 方法");
        }

        // 属性：每个路由都应该有处理器函数名
        for route in &navigator.index.routes {
            prop_assert!(!route.handler.function_name.is_empty(), "路由应该有处理器函数名");
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 36: 路由完整识别（空文档）
    ///
    /// 对于没有路由宏的文档，路由导航器应该返回空索引。
    #[test]
    fn prop_route_recognition_empty_documents(_dummy in 0..100u32) {
        let mut navigator = RouteNavigator::new();
        let documents = vec![];

        navigator.build_index(&documents);

        // 属性：空文档应该产生空索引
        prop_assert_eq!(navigator.index.routes.len(), 0);
        prop_assert_eq!(navigator.index.path_map.len(), 0);
    }
}

// ============================================================================
// Property 37: 路径参数解析
// **Validates: Requirements 8.2**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 37: 路径参数解析
    ///
    /// For any 包含路径参数（如 `{id}`）的路由，路由导航器应该正确解析参数名称。
    #[test]
    fn prop_path_parameter_parsing(
        (path, expected_params) in route_path_with_params(),
        methods in multiple_http_methods(),
        handler_name in valid_handler_name(),
        range in valid_range(),
        uri in valid_file_uri()
    ) {
        let mut navigator = RouteNavigator::new();

        let route_macro = RouteMacro {
            path: path.clone(),
            methods: methods.clone(),
            middlewares: vec![],
            handler_name: handler_name.clone(),
            range,
        };

        let doc = RustDocument {
            uri,
            content: String::new(),
            macros: vec![SpringMacro::Route(route_macro)],
        };

        navigator.build_index(&[doc]);

        // 属性：应该为每个方法创建一个路由条目
        prop_assert_eq!(navigator.index.routes.len(), methods.len());

        // 属性：每个路由条目都应该正确解析路径参数
        for route in &navigator.index.routes {
            prop_assert_eq!(&route.path, &path, "路由路径应该匹配");

            // 验证参数数量
            prop_assert_eq!(
                route.handler.parameters.len(),
                expected_params.len(),
                "应该解析出正确数量的路径参数"
            );

            // 验证参数名称
            for (i, expected_param) in expected_params.iter().enumerate() {
                prop_assert_eq!(
                    &route.handler.parameters[i].name,
                    expected_param,
                    "参数名称应该匹配"
                );
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 37: 路径参数解析（无参数）
    ///
    /// 对于没有路径参数的路由，应该返回空参数列表。
    #[test]
    fn prop_path_parameter_parsing_no_params(
        methods in multiple_http_methods(),
        handler_name in valid_handler_name(),
        range in valid_range(),
        uri in valid_file_uri()
    ) {
        let mut navigator = RouteNavigator::new();

        let paths_without_params = vec![
            "/users",
            "/api/v1/users",
            "/posts",
            "/health",
            "/",
        ];

        for path in paths_without_params {
            let route_macro = RouteMacro {
                path: path.to_string(),
                methods: methods.clone(),
                middlewares: vec![],
                handler_name: handler_name.clone(),
                range,
            };

            let doc = RustDocument {
                uri: uri.clone(),
                content: String::new(),
                macros: vec![SpringMacro::Route(route_macro)],
            };

            navigator.build_index(&[doc]);

            // 属性：没有路径参数的路由应该有空参数列表
            for route in &navigator.index.routes {
                prop_assert_eq!(
                    route.handler.parameters.len(),
                    0,
                    "没有路径参数的路由应该有空参数列表"
                );
            }

            // 清空索引以便下一次测试
            navigator.index.routes.clear();
            navigator.index.path_map.clear();
        }
    }
}

// ============================================================================
// Property 38: 多方法路由展开
// **Validates: Requirements 8.3**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 38: 多方法路由展开
    ///
    /// For any 使用 `#[route]` 宏指定多个 HTTP 方法的路由，
    /// 路由导航器应该为每个方法创建独立的路由条目。
    #[test]
    fn prop_multi_method_route_expansion(
        path in valid_route_path(),
        methods in multiple_http_methods(),
        handler_name in valid_handler_name(),
        range in valid_range(),
        uri in valid_file_uri()
    ) {
        // 确保至少有 2 个方法
        prop_assume!(methods.len() >= 2);

        let mut navigator = RouteNavigator::new();

        let route_macro = RouteMacro {
            path: path.clone(),
            methods: methods.clone(),
            middlewares: vec![],
            handler_name: handler_name.clone(),
            range,
        };

        let doc = RustDocument {
            uri,
            content: String::new(),
            macros: vec![SpringMacro::Route(route_macro)],
        };

        navigator.build_index(&[doc]);

        // 属性：应该为每个方法创建独立的路由条目
        prop_assert_eq!(
            navigator.index.routes.len(),
            methods.len(),
            "应该为每个 HTTP 方法创建独立的路由条目"
        );

        // 属性：每个路由条目应该只有一个方法
        for route in &navigator.index.routes {
            prop_assert_eq!(
                route.methods.len(),
                1,
                "每个路由条目应该只有一个 HTTP 方法"
            );
        }

        // 属性：所有路由条目应该有相同的路径
        for route in &navigator.index.routes {
            prop_assert_eq!(
                &route.path,
                &path,
                "所有路由条目应该有相同的路径"
            );
        }

        // 属性：所有路由条目应该有相同的处理器函数名
        for route in &navigator.index.routes {
            prop_assert_eq!(
                &route.handler.function_name,
                &handler_name,
                "所有路由条目应该有相同的处理器函数名"
            );
        }

        // 属性：所有方法都应该出现在路由条目中
        let route_methods: Vec<_> = navigator.index.routes
            .iter()
            .map(|r| r.methods[0].clone())
            .collect();

        for method in &methods {
            prop_assert!(
                route_methods.contains(method),
                "所有方法都应该出现在路由条目中"
            );
        }

        // 属性：路径映射应该包含所有路由条目的索引
        let path_indices = navigator.index.path_map.get(&path);
        prop_assert!(path_indices.is_some(), "路径映射应该包含该路径");

        let path_indices = path_indices.unwrap();
        prop_assert_eq!(
            path_indices.len(),
            methods.len(),
            "路径映射应该包含所有路由条目的索引"
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 38: 多方法路由展开（单方法）
    ///
    /// 对于只有一个方法的路由，应该创建一个路由条目。
    #[test]
    fn prop_single_method_route_no_expansion(
        path in valid_route_path(),
        method in single_http_method(),
        handler_name in valid_handler_name(),
        range in valid_range(),
        uri in valid_file_uri()
    ) {
        let mut navigator = RouteNavigator::new();

        let route_macro = RouteMacro {
            path: path.clone(),
            methods: vec![method.clone()],
            middlewares: vec![],
            handler_name: handler_name.clone(),
            range,
        };

        let doc = RustDocument {
            uri,
            content: String::new(),
            macros: vec![SpringMacro::Route(route_macro)],
        };

        navigator.build_index(&[doc]);

        // 属性：单方法路由应该只创建一个路由条目
        prop_assert_eq!(navigator.index.routes.len(), 1);

        // 属性：路由条目应该有正确的方法
        prop_assert_eq!(navigator.index.routes[0].methods.len(), 1);
        prop_assert_eq!(&navigator.index.routes[0].methods[0], &method);
    }
}

// ============================================================================
// Property 39: 路由前缀解析
// **Validates: Requirements 8.4**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 39: 路由前缀解析
    ///
    /// For any 包含路由前缀配置的路由，路由导航器应该解析并返回完整的路由路径（前缀 + 路径）。
    ///
    /// 注意：当前实现中，路由前缀的解析需要在 MacroAnalyzer 中实现。
    /// 这个测试验证路由导航器能够正确处理已经包含前缀的完整路径。
    #[test]
    fn prop_route_prefix_parsing(
        prefix in prop::sample::select(vec!["/api", "/api/v1", "/api/v2", ""]),
        base_path in prop::sample::select(vec!["/users", "/posts", "/comments"]),
        methods in multiple_http_methods(),
        handler_name in valid_handler_name(),
        range in valid_range(),
        uri in valid_file_uri()
    ) {
        let mut navigator = RouteNavigator::new();

        // 构建完整路径（前缀 + 基础路径）
        let full_path = if prefix.is_empty() {
            base_path.to_string()
        } else {
            format!("{}{}", prefix, base_path)
        };

        let route_macro = RouteMacro {
            path: full_path.clone(),
            methods: methods.clone(),
            middlewares: vec![],
            handler_name: handler_name.clone(),
            range,
        };

        let doc = RustDocument {
            uri,
            content: String::new(),
            macros: vec![SpringMacro::Route(route_macro)],
        };

        navigator.build_index(&[doc]);

        // 属性：路由应该使用完整路径（包含前缀）
        for route in &navigator.index.routes {
            prop_assert_eq!(
                &route.path,
                &full_path,
                "路由应该使用完整路径（包含前缀）"
            );
        }

        // 属性：路径映射应该使用完整路径作为键
        prop_assert!(
            navigator.index.path_map.contains_key(&full_path),
            "路径映射应该使用完整路径作为键"
        );

        // 属性：如果有前缀，路径应该以前缀开头
        if !prefix.is_empty() {
            for route in &navigator.index.routes {
                prop_assert!(
                    route.path.starts_with(prefix),
                    "路径应该以前缀开头"
                );
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 39: 路由前缀解析（无前缀）
    ///
    /// 对于没有前缀的路由，应该使用原始路径。
    #[test]
    fn prop_route_no_prefix(
        path in valid_route_path(),
        methods in multiple_http_methods(),
        handler_name in valid_handler_name(),
        range in valid_range(),
        uri in valid_file_uri()
    ) {
        let mut navigator = RouteNavigator::new();

        let route_macro = RouteMacro {
            path: path.clone(),
            methods: methods.clone(),
            middlewares: vec![],
            handler_name: handler_name.clone(),
            range,
        };

        let doc = RustDocument {
            uri,
            content: String::new(),
            macros: vec![SpringMacro::Route(route_macro)],
        };

        navigator.build_index(&[doc]);

        // 属性：没有前缀的路由应该使用原始路径
        for route in &navigator.index.routes {
            prop_assert_eq!(
                &route.path,
                &path,
                "没有前缀的路由应该使用原始路径"
            );
        }
    }
}

// ============================================================================
// 额外的属性测试：验证索引的一致性
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// 验证路径映射的一致性
    ///
    /// 路径映射中的索引应该指向有效的路由条目，且路由条目的路径应该与映射的键匹配。
    #[test]
    fn prop_path_map_consistency(
        documents in multiple_rust_documents()
    ) {
        let mut navigator = RouteNavigator::new();
        navigator.build_index(&documents);

        // 属性：路径映射中的每个索引都应该指向有效的路由
        for (path, indices) in &navigator.index.path_map {
            for &index in indices {
                prop_assert!(
                    index < navigator.index.routes.len(),
                    "路径映射中的索引应该指向有效的路由"
                );

                // 属性：路由的路径应该与映射的键匹配
                prop_assert_eq!(
                    &navigator.index.routes[index].path,
                    path,
                    "路由的路径应该与映射的键匹配"
                );
            }
        }

        // 属性：每个路由都应该在路径映射中
        for (i, route) in navigator.index.routes.iter().enumerate() {
            let indices = navigator.index.path_map.get(&route.path);
            prop_assert!(
                indices.is_some(),
                "每个路由都应该在路径映射中"
            );

            let indices = indices.unwrap();
            prop_assert!(
                indices.contains(&i),
                "路径映射应该包含该路由的索引"
            );
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// 验证重建索引的正确性
    ///
    /// 重建索引应该清空旧索引并构建新索引。
    #[test]
    fn prop_rebuild_index_clears_old_data(
        documents1 in multiple_rust_documents(),
        documents2 in multiple_rust_documents()
    ) {
        let mut navigator = RouteNavigator::new();

        // 第一次构建
        navigator.build_index(&documents1);
        let _first_count = navigator.index.routes.len();

        // 第二次构建
        navigator.build_index(&documents2);
        let second_count = navigator.index.routes.len();

        // 计算第二次构建的预期路由数量
        let expected_count: usize = documents2
            .iter()
            .flat_map(|doc| &doc.macros)
            .filter_map(|m| {
                if let SpringMacro::Route(route) = m {
                    Some(route.methods.len())
                } else {
                    None
                }
            })
            .sum();

        // 属性：第二次构建应该只包含新文档的路由
        prop_assert_eq!(
            second_count,
            expected_count,
            "重建索引应该只包含新文档的路由"
        );

        // 属性：如果两次文档不同，路由数量可能不同
        // （这不是一个强属性，只是一个观察）
        if documents1.len() != documents2.len() {
            // 可能不同，但不一定
        }
    }
}
