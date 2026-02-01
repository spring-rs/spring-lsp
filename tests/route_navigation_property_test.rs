//! 路由导航属性测试
//!
//! 使用 proptest 验证路由导航和查找功能在随机生成的输入下的正确性

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
// Property 40: 路由列表完整性
// **Validates: Requirements 9.1**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 40: 路由列表完整性
    ///
    /// For any 项目，路由导航器返回的路由列表应该包含所有已识别的路由。
    #[test]
    fn prop_route_list_completeness(
        documents in multiple_rust_documents()
    ) {
        let mut navigator = RouteNavigator::new();

        // 构建索引
        navigator.build_index(&documents);

        // 获取所有路由
        let all_routes = navigator.get_all_routes();

        // 计算预期的路由数量
        let expected_count: usize = documents
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

        // 属性：返回的路由列表应该包含所有已识别的路由
        prop_assert_eq!(
            all_routes.len(),
            expected_count,
            "路由列表应该包含所有已识别的路由"
        );

        // 属性：每个路由都应该有有效的数据
        for route in all_routes {
            prop_assert!(!route.path.is_empty(), "路由路径不应为空");
            prop_assert!(!route.methods.is_empty(), "路由应该至少有一个 HTTP 方法");
            prop_assert!(!route.handler.function_name.is_empty(), "路由应该有处理器函数名");
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 40: 路由列表完整性（空索引）
    ///
    /// 对于空索引，应该返回空列表。
    #[test]
    fn prop_route_list_empty(_dummy in 0..100u32) {
        let navigator = RouteNavigator::new();
        let all_routes = navigator.get_all_routes();

        // 属性：空索引应该返回空列表
        prop_assert_eq!(all_routes.len(), 0);
    }
}

// ============================================================================
// Property 41: 路由跳转正确性
// **Validates: Requirements 9.2**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 41: 路由跳转正确性
    ///
    /// For any 路由路径，点击跳转应该定位到定义该路由的处理器函数。
    #[test]
    fn prop_route_jump_correctness(
        documents in multiple_rust_documents()
    ) {
        let mut navigator = RouteNavigator::new();

        // 构建索引
        navigator.build_index(&documents);

        // 属性：每个路由的 location 应该指向有效的文件和位置
        for route in navigator.get_all_routes() {
            // 验证 URI 有效
            prop_assert!(route.location.uri.scheme() == "file", "URI 应该是 file 协议");

            // 验证范围有效
            prop_assert!(
                route.location.range.start.line <= route.location.range.end.line,
                "起始行应该小于等于结束行"
            );

            // 如果在同一行，起始字符应该小于等于结束字符
            if route.location.range.start.line == route.location.range.end.line {
                prop_assert!(
                    route.location.range.start.character <= route.location.range.end.character,
                    "同一行时，起始字符应该小于等于结束字符"
                );
            }
        }
    }
}

// ============================================================================
// Property 42: 处理器路由反查
// **Validates: Requirements 9.3**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 42: 处理器路由反查
    ///
    /// For any 路由处理器函数，应该能够查询到该函数对应的所有路由信息。
    #[test]
    fn prop_handler_route_reverse_lookup(
        documents in multiple_rust_documents()
    ) {
        let mut navigator = RouteNavigator::new();

        // 构建索引
        navigator.build_index(&documents);

        // 收集所有处理器名称
        let handler_names: Vec<String> = documents
            .iter()
            .flat_map(|doc| &doc.macros)
            .filter_map(|m| {
                if let SpringMacro::Route(route) = m {
                    Some(route.handler_name.clone())
                } else {
                    None
                }
            })
            .collect();

        // 属性：对于每个处理器，应该能够找到对应的路由
        for handler_name in handler_names.iter() {
            let routes = navigator.find_routes_by_handler(handler_name);

            // 应该至少找到一个路由
            prop_assert!(
                !routes.is_empty(),
                "应该能够找到处理器 {} 对应的路由",
                handler_name
            );

            // 所有找到的路由都应该有正确的处理器名称
            for route in routes {
                prop_assert_eq!(
                    &route.handler.function_name,
                    handler_name,
                    "路由的处理器名称应该匹配"
                );
            }
        }

        // 属性：查找不存在的处理器应该返回空列表
        let non_existent_handler = "non_existent_handler_xyz_123";
        let routes = navigator.find_routes_by_handler(non_existent_handler);
        prop_assert_eq!(routes.len(), 0, "不存在的处理器应该返回空列表");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 42: 处理器路由反查（多方法路由）
    ///
    /// 对于多方法路由，应该为每个方法返回独立的路由条目。
    #[test]
    fn prop_handler_reverse_lookup_multi_method(
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

        // 属性：应该找到与方法数量相同的路由条目
        let routes = navigator.find_routes_by_handler(&handler_name);
        prop_assert_eq!(
            routes.len(),
            methods.len(),
            "应该为每个方法返回独立的路由条目"
        );

        // 属性：所有路由都应该有相同的路径和处理器名称
        for route in routes {
            prop_assert_eq!(&route.path, &path);
            prop_assert_eq!(&route.handler.function_name, &handler_name);
        }
    }
}

// ============================================================================
// Property 43: 路由搜索匹配
// **Validates: Requirements 9.4**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 43: 路由搜索匹配（模糊匹配）
    ///
    /// For any 路由搜索模式（模糊匹配），搜索结果应该包含所有匹配该模式的路由。
    #[test]
    fn prop_route_search_fuzzy_match(
        documents in multiple_rust_documents()
    ) {
        let mut navigator = RouteNavigator::new();

        // 构建索引
        navigator.build_index(&documents);

        // 收集所有路由路径
        let all_routes = navigator.get_all_routes();

        // 测试不同的搜索模式
        let search_patterns = vec!["users", "api", "posts", "/", "{id}"];

        for pattern in search_patterns {
            let found_routes = navigator.find_routes(pattern);

            // 属性：所有找到的路由都应该包含搜索模式
            for route in &found_routes {
                prop_assert!(
                    route.path.contains(pattern),
                    "路由路径 {} 应该包含搜索模式 {}",
                    route.path,
                    pattern
                );
            }

            // 属性：所有包含搜索模式的路由都应该被找到
            let expected_routes: Vec<_> = all_routes
                .iter()
                .filter(|r| r.path.contains(pattern))
                .collect();

            prop_assert_eq!(
                found_routes.len(),
                expected_routes.len(),
                "应该找到所有包含搜索模式的路由"
            );
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 43: 路由搜索匹配（正则表达式）
    ///
    /// For any 路由搜索模式（正则表达式），搜索结果应该包含所有匹配该模式的路由。
    #[test]
    fn prop_route_search_regex_match(
        documents in multiple_rust_documents()
    ) {
        let mut navigator = RouteNavigator::new();

        // 构建索引
        navigator.build_index(&documents);

        // 测试不同的正则表达式模式
        let regex_patterns = vec![
            "regex:^/api/.*",      // 以 /api/ 开头
            "regex:.*\\{.*\\}.*",  // 包含参数
            "regex:^/users",       // 以 /users 开头
            "regex:.*/posts.*",    // 包含 /posts
        ];

        for pattern in regex_patterns {
            let found_routes = navigator.find_routes(pattern);

            // 提取正则表达式部分
            let regex_str = pattern.strip_prefix("regex:").unwrap();

            // 如果正则表达式有效，验证匹配结果
            if let Ok(re) = regex::Regex::new(regex_str) {
                // 属性：所有找到的路由都应该匹配正则表达式
                for route in &found_routes {
                    prop_assert!(
                        re.is_match(&route.path),
                        "路由路径 {} 应该匹配正则表达式 {}",
                        route.path,
                        regex_str
                    );
                }

                // 属性：所有匹配正则表达式的路由都应该被找到
                let all_routes = navigator.get_all_routes();
                let expected_routes: Vec<_> = all_routes
                    .iter()
                    .filter(|r| re.is_match(&r.path))
                    .collect();

                prop_assert_eq!(
                    found_routes.len(),
                    expected_routes.len(),
                    "应该找到所有匹配正则表达式的路由"
                );
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 43: 路由搜索匹配（空模式）
    ///
    /// 空搜索模式应该返回空列表。
    #[test]
    fn prop_route_search_empty_pattern(
        documents in multiple_rust_documents()
    ) {
        let mut navigator = RouteNavigator::new();

        // 构建索引
        navigator.build_index(&documents);

        // 属性：空模式应该返回空列表
        let routes = navigator.find_routes("");
        prop_assert_eq!(routes.len(), 0, "空模式应该返回空列表");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 43: 路由搜索匹配（无效正则表达式）
    ///
    /// 无效的正则表达式应该返回空列表。
    #[test]
    fn prop_route_search_invalid_regex(
        documents in multiple_rust_documents()
    ) {
        let mut navigator = RouteNavigator::new();

        // 构建索引
        navigator.build_index(&documents);

        // 测试无效的正则表达式
        let invalid_patterns = vec![
            "regex:[invalid",
            "regex:(?P<",
            "regex:*",
            "regex:(?",
        ];

        for pattern in invalid_patterns {
            let routes = navigator.find_routes(pattern);

            // 属性：无效的正则表达式应该返回空列表
            prop_assert_eq!(
                routes.len(),
                0,
                "无效的正则表达式 {} 应该返回空列表",
                pattern
            );
        }
    }
}

// ============================================================================
// 额外的属性测试：验证搜索结果的一致性
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// 验证搜索结果的一致性
    ///
    /// 搜索结果应该是索引中路由的子集。
    #[test]
    fn prop_search_results_consistency(
        documents in multiple_rust_documents()
    ) {
        let mut navigator = RouteNavigator::new();

        // 构建索引
        navigator.build_index(&documents);

        let all_routes = navigator.get_all_routes();

        // 测试不同的搜索模式
        let patterns = vec!["users", "api", "posts", "regex:^/.*"];

        for pattern in patterns {
            let found_routes = navigator.find_routes(pattern);

            // 属性：所有找到的路由都应该在索引中
            for found_route in &found_routes {
                let found_in_index = all_routes.iter().any(|r| {
                    r.path == found_route.path
                        && r.handler.function_name == found_route.handler.function_name
                        && r.methods == found_route.methods
                });

                prop_assert!(
                    found_in_index,
                    "搜索结果应该是索引中路由的子集"
                );
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// 验证处理器反查的一致性
    ///
    /// 处理器反查的结果应该与直接遍历索引的结果一致。
    #[test]
    fn prop_handler_lookup_consistency(
        documents in multiple_rust_documents()
    ) {
        let mut navigator = RouteNavigator::new();

        // 构建索引
        navigator.build_index(&documents);

        // 收集所有处理器名称
        let handler_names: Vec<String> = documents
            .iter()
            .flat_map(|doc| &doc.macros)
            .filter_map(|m| {
                if let SpringMacro::Route(route) = m {
                    Some(route.handler_name.clone())
                } else {
                    None
                }
            })
            .collect();

        for handler_name in handler_names.iter() {
            // 使用反查方法
            let found_by_lookup = navigator.find_routes_by_handler(handler_name);

            // 直接遍历索引
            let found_by_iteration: Vec<_> = navigator
                .get_all_routes()
                .iter()
                .filter(|r| r.handler.function_name == *handler_name)
                .collect();

            // 属性：两种方法应该返回相同数量的路由
            prop_assert_eq!(
                found_by_lookup.len(),
                found_by_iteration.len(),
                "反查方法和直接遍历应该返回相同数量的路由"
            );

            // 属性：两种方法返回的路由应该相同
            for route in &found_by_lookup {
                let found = found_by_iteration.iter().any(|r| {
                    r.path == route.path
                        && r.handler.function_name == route.handler.function_name
                        && r.methods == route.methods
                });

                prop_assert!(
                    found,
                    "反查方法和直接遍历应该返回相同的路由"
                );
            }
        }
    }
}
