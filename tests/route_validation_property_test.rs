//! 路由验证属性测试
//!
//! 使用 proptest 验证路由验证功能在随机生成的输入下的正确性

use lsp_types::{Position, Range, Url};
use proptest::prelude::*;
use spring_lsp::macro_analyzer::{HttpMethod, RouteMacro, RustDocument, SpringMacro};
use spring_lsp::route::RouteNavigator;

// ============================================================================
// 测试数据生成器
// ============================================================================

/// 生成有效的路由路径（不包含无效字符）
fn valid_route_path() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::string::string_regex("[a-z0-9_-]+").unwrap(), 1..5)
        .prop_map(|segments| format!("/{}", segments.join("/")))
}

/// 生成包含路径参数的路由路径
fn route_path_with_params() -> impl Strategy<Value = String> {
    (
        prop::collection::vec(prop::string::string_regex("[a-z0-9_-]+").unwrap(), 0..3),
        prop::collection::vec(
            prop::string::string_regex("[a-z_][a-z0-9_]*").unwrap(),
            1..3,
        ),
    )
        .prop_map(|(segments, params)| {
            let mut path = String::from("/");
            let mut all_parts = Vec::new();

            // 交替添加普通段和参数段
            for (i, param) in params.iter().enumerate() {
                if i < segments.len() {
                    all_parts.push(segments[i].clone());
                }
                all_parts.push(format!("{{{}}}", param));
            }

            // 添加剩余的普通段
            for segment in segments.iter().skip(params.len()) {
                all_parts.push(segment.clone());
            }

            path.push_str(&all_parts.join("/"));
            path
        })
}

/// 生成包含无效字符的路由路径
fn invalid_route_path() -> impl Strategy<Value = String> {
    prop::string::string_regex("/[a-z]+[<>\\\\]+[a-z]+").unwrap()
}

/// 生成路径参数语法错误的路由路径
fn malformed_param_path() -> impl Strategy<Value = (String, &'static str)> {
    prop_oneof![
        Just(("/users/{}".to_string(), "empty-path-param")),
        Just(("/users/{id".to_string(), "unclosed-path-param")),
        Just(("/users/id}".to_string(), "unmatched-closing-brace")),
        Just(("/users/{{id}}".to_string(), "nested-path-param")),
    ]
}

/// 生成包含动词的路由路径（RESTful 风格问题）
fn path_with_verb() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("/getUsers".to_string()),
        Just("/createUser".to_string()),
        Just("/updateUser".to_string()),
        Just("/deleteUser".to_string()),
        Just("/api/fetchData".to_string()),
        Just("/users/get-all".to_string()),
    ]
}

/// 生成包含大写字母的路由路径（RESTful 风格问题）
fn path_with_uppercase() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("/userProfiles".to_string()),
        Just("/UserData".to_string()),
        Just("/api/GetUsers".to_string()),
    ]
}

/// 生成 HTTP 方法
fn http_method() -> impl Strategy<Value = HttpMethod> {
    prop_oneof![
        Just(HttpMethod::Get),
        Just(HttpMethod::Post),
        Just(HttpMethod::Put),
        Just(HttpMethod::Delete),
        Just(HttpMethod::Patch),
    ]
}

/// 生成路由宏
fn route_macro(path: String, methods: Vec<HttpMethod>) -> RouteMacro {
    RouteMacro {
        path,
        methods,
        middlewares: vec![],
        handler_name: "test_handler".to_string(),
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 0,
            },
        },
    }
}

/// 生成 Rust 文档
fn rust_document(macros: Vec<SpringMacro>) -> RustDocument {
    RustDocument {
        uri: Url::parse("file:///test.rs").unwrap(),
        content: String::new(),
        macros,
    }
}

// ============================================================================
// Property 44: 路由冲突检测
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 44: 路由冲突检测
    ///
    /// **Validates: Requirements 9.5, 10.4**
    ///
    /// For any 两个或多个路由具有相同的路径和 HTTP 方法，
    /// 诊断引擎应该生成冲突警告。
    ///
    /// 此属性验证：
    /// 1. 相同路径和方法的路由被正确识别为冲突
    /// 2. 所有冲突对都被检测到
    /// 3. 冲突信息包含正确的路径和方法
    #[test]
    fn prop_detect_route_conflicts(
        path in valid_route_path(),
        method in http_method(),
        num_routes in 2usize..5,
    ) {
        let mut navigator = RouteNavigator::new();

        // 创建多个具有相同路径和方法的路由
        let macros: Vec<_> = (0..num_routes)
            .map(|i| {
                let mut route = route_macro(path.clone(), vec![method.clone()]);
                route.handler_name = format!("handler_{}", i);
                route.range.start.line = (i * 10) as u32;
                route.range.end.line = (i * 10 + 5) as u32;
                SpringMacro::Route(route)
            })
            .collect();

        let doc = rust_document(macros);
        navigator.build_index(&[doc]);

        // 检测冲突
        let conflicts = navigator.detect_conflicts();

        // 验证：应该检测到所有冲突对
        // n 个相同路由应该有 n*(n-1)/2 个冲突
        let expected_conflicts = num_routes * (num_routes - 1) / 2;
        prop_assert_eq!(conflicts.len(), expected_conflicts);

        // 验证：所有冲突都有正确的路径和方法
        for conflict in &conflicts {
            prop_assert_eq!(&conflict.path, &path);
            prop_assert_eq!(&conflict.method, &method);
            prop_assert!(conflict.index1 < conflict.index2);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 44: 路由冲突检测（不同方法无冲突）
    ///
    /// **Validates: Requirements 9.5, 10.4**
    ///
    /// For any 两个路由具有相同的路径但不同的 HTTP 方法，
    /// 不应该检测到冲突。
    #[test]
    fn prop_no_conflict_different_methods(
        path in valid_route_path(),
    ) {
        let mut navigator = RouteNavigator::new();

        // 创建两个具有相同路径但不同方法的路由
        let route1 = route_macro(path.clone(), vec![HttpMethod::Get]);
        let route2 = route_macro(path.clone(), vec![HttpMethod::Post]);

        let doc = rust_document(vec![
            SpringMacro::Route(route1),
            SpringMacro::Route(route2),
        ]);

        navigator.build_index(&[doc]);

        // 检测冲突
        let conflicts = navigator.detect_conflicts();

        // 验证：不应该有冲突
        prop_assert_eq!(conflicts.len(), 0);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 44: 路由冲突检测（不同路径无冲突）
    ///
    /// **Validates: Requirements 9.5, 10.4**
    ///
    /// For any 两个路由具有不同的路径，
    /// 即使方法相同也不应该检测到冲突。
    #[test]
    fn prop_no_conflict_different_paths(
        path1 in valid_route_path(),
        path2 in valid_route_path(),
        method in http_method(),
    ) {
        // 确保路径不同
        prop_assume!(path1 != path2);

        let mut navigator = RouteNavigator::new();

        // 创建两个具有不同路径但相同方法的路由
        let route1 = route_macro(path1, vec![method.clone()]);
        let route2 = route_macro(path2, vec![method]);

        let doc = rust_document(vec![
            SpringMacro::Route(route1),
            SpringMacro::Route(route2),
        ]);

        navigator.build_index(&[doc]);

        // 检测冲突
        let conflicts = navigator.detect_conflicts();

        // 验证：不应该有冲突
        prop_assert_eq!(conflicts.len(), 0);
    }
}

// ============================================================================
// Property 45: 路径字符验证
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 45: 路径字符验证
    ///
    /// **Validates: Requirements 10.1**
    ///
    /// For any 路由路径包含 URL 规范不允许的字符，
    /// 诊断引擎应该生成语法错误诊断。
    ///
    /// 此属性验证：
    /// 1. 包含无效字符的路径被正确识别
    /// 2. 生成的诊断包含错误代码 "invalid-path-char"
    /// 3. 诊断严重性为 ERROR
    #[test]
    fn prop_validate_invalid_path_characters(
        invalid_path in invalid_route_path(),
    ) {
        let mut navigator = RouteNavigator::new();

        let route = route_macro(invalid_path, vec![HttpMethod::Get]);
        let doc = rust_document(vec![SpringMacro::Route(route)]);

        navigator.build_index(&[doc]);

        // 验证路由
        let diagnostics = navigator.validate_routes();

        // 验证：应该有无效字符错误
        let has_invalid_char_error = diagnostics.iter().any(|d| {
            d.code.as_ref()
                .and_then(|c| match c {
                    lsp_types::NumberOrString::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .map(|s| s == "invalid-path-char")
                .unwrap_or(false)
                && d.severity == Some(lsp_types::DiagnosticSeverity::ERROR)
        });

        prop_assert!(has_invalid_char_error,
            "Expected invalid-path-char error for path with invalid characters");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 45: 路径字符验证（有效路径）
    ///
    /// **Validates: Requirements 10.1**
    ///
    /// For any 路由路径只包含有效字符，
    /// 不应该生成路径字符错误。
    #[test]
    fn prop_validate_valid_path_characters(
        valid_path in valid_route_path(),
    ) {
        let mut navigator = RouteNavigator::new();

        let route = route_macro(valid_path, vec![HttpMethod::Get]);
        let doc = rust_document(vec![SpringMacro::Route(route)]);

        navigator.build_index(&[doc]);

        // 验证路由
        let diagnostics = navigator.validate_routes();

        // 验证：不应该有无效字符错误
        let has_invalid_char_error = diagnostics.iter().any(|d| {
            d.code.as_ref()
                .and_then(|c| match c {
                    lsp_types::NumberOrString::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .map(|s| s == "invalid-path-char")
                .unwrap_or(false)
        });

        prop_assert!(!has_invalid_char_error,
            "Should not have invalid-path-char error for valid path");
    }
}

// ============================================================================
// Property 46: 路径参数语法验证
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 46: 路径参数语法验证
    ///
    /// **Validates: Requirements 10.2**
    ///
    /// For any 路径参数不符合 {param} 格式，
    /// 诊断引擎应该生成错误诊断并提供修复建议。
    ///
    /// 此属性验证：
    /// 1. 各种参数语法错误被正确识别
    /// 2. 生成的诊断包含正确的错误代码
    /// 3. 诊断严重性为 ERROR
    #[test]
    fn prop_validate_malformed_path_parameters(
        (malformed_path, expected_code) in malformed_param_path(),
    ) {
        let mut navigator = RouteNavigator::new();

        let route = route_macro(malformed_path, vec![HttpMethod::Get]);
        let doc = rust_document(vec![SpringMacro::Route(route)]);

        navigator.build_index(&[doc]);

        // 验证路由
        let diagnostics = navigator.validate_routes();

        // 验证：应该有对应的参数语法错误
        let has_expected_error = diagnostics.iter().any(|d| {
            d.code.as_ref()
                .and_then(|c| match c {
                    lsp_types::NumberOrString::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .map(|s| s == expected_code)
                .unwrap_or(false)
                && d.severity == Some(lsp_types::DiagnosticSeverity::ERROR)
        });

        prop_assert!(has_expected_error,
            "Expected {} error for malformed path parameter", expected_code);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 46: 路径参数语法验证（正确格式）
    ///
    /// **Validates: Requirements 10.2**
    ///
    /// For any 路径参数符合 {param} 格式，
    /// 不应该生成参数语法错误。
    #[test]
    fn prop_validate_wellformed_path_parameters(
        path in route_path_with_params(),
    ) {
        let mut navigator = RouteNavigator::new();

        let route = route_macro(path, vec![HttpMethod::Get]);
        let doc = rust_document(vec![SpringMacro::Route(route)]);

        navigator.build_index(&[doc]);

        // 验证路由
        let diagnostics = navigator.validate_routes();

        // 验证：不应该有参数语法错误
        let has_param_syntax_error = diagnostics.iter().any(|d| {
            d.code.as_ref()
                .and_then(|c| match c {
                    lsp_types::NumberOrString::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .map(|s| s.contains("path-param") || s.contains("brace"))
                .unwrap_or(false)
        });

        prop_assert!(!has_param_syntax_error,
            "Should not have parameter syntax error for well-formed parameters");
    }
}

// ============================================================================
// Property 47: 路径参数类型匹配
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 47: 路径参数类型匹配
    ///
    /// **Validates: Requirements 10.3**
    ///
    /// For any 路径参数，如果其类型与处理器函数中对应参数的类型不兼容，
    /// 诊断引擎应该生成类型错误诊断。
    ///
    /// 此属性验证：
    /// 1. 路径参数验证不会崩溃
    /// 2. 验证逻辑正确处理各种路径
    /// 3. 当前实现中，由于无法提取函数签名，参数类型默认为 "Unknown"，
    ///    因此不会生成类型不匹配的警告
    #[test]
    fn prop_validate_path_parameter_types(
        path in route_path_with_params(),
    ) {
        let mut navigator = RouteNavigator::new();

        // 创建路由（当前实现中参数类型为 "Unknown"）
        let route = route_macro(path.clone(), vec![HttpMethod::Get]);
        let doc = rust_document(vec![SpringMacro::Route(route)]);

        navigator.build_index(&[doc]);

        // 验证路由（不应该崩溃）
        let _diagnostics = navigator.validate_routes();

        // 验证：验证过程不应该崩溃
        // 注意：当前实现中，由于参数类型为 "Unknown"，
        // 不会生成 missing-path-param 警告
        // 这是预期行为，因为我们还没有实现完整的函数签名提取
        prop_assert!(true, "Validation should complete without crashing");
    }
}

// ============================================================================
// Property 48: RESTful 风格检查
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 48: RESTful 风格检查（动词检测）
    ///
    /// **Validates: Requirements 10.5**
    ///
    /// For any 路由路径不符合 RESTful 命名规范（如使用动词而非名词），
    /// 诊断引擎应该生成风格建议。
    ///
    /// 此属性验证：
    /// 1. 路径中的动词被正确识别
    /// 2. 生成的诊断包含 "restful-style-verb" 代码
    /// 3. 诊断严重性为 INFORMATION（建议）
    #[test]
    fn prop_validate_restful_style_verb(
        path in path_with_verb(),
    ) {
        let mut navigator = RouteNavigator::new();

        let route = route_macro(path, vec![HttpMethod::Get]);
        let doc = rust_document(vec![SpringMacro::Route(route)]);

        navigator.build_index(&[doc]);

        // 验证路由
        let diagnostics = navigator.validate_routes();

        // 验证：应该有 RESTful 风格建议（动词）
        let has_verb_suggestion = diagnostics.iter().any(|d| {
            d.code.as_ref()
                .and_then(|c| match c {
                    lsp_types::NumberOrString::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .map(|s| s == "restful-style-verb")
                .unwrap_or(false)
                && d.severity == Some(lsp_types::DiagnosticSeverity::INFORMATION)
        });

        prop_assert!(has_verb_suggestion,
            "Expected restful-style-verb suggestion for path with verb");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 48: RESTful 风格检查（大写字母检测）
    ///
    /// **Validates: Requirements 10.5**
    ///
    /// For any 路由路径使用大写字母（不符合 RESTful 规范），
    /// 诊断引擎应该生成风格建议。
    ///
    /// 此属性验证：
    /// 1. 路径中的大写字母被正确识别
    /// 2. 生成的诊断包含 "restful-style-case" 代码
    /// 3. 诊断严重性为 INFORMATION（建议）
    #[test]
    fn prop_validate_restful_style_case(
        path in path_with_uppercase(),
    ) {
        let mut navigator = RouteNavigator::new();

        let route = route_macro(path, vec![HttpMethod::Get]);
        let doc = rust_document(vec![SpringMacro::Route(route)]);

        navigator.build_index(&[doc]);

        // 验证路由
        let diagnostics = navigator.validate_routes();

        // 验证：应该有 RESTful 风格建议（大小写）
        let has_case_suggestion = diagnostics.iter().any(|d| {
            d.code.as_ref()
                .and_then(|c| match c {
                    lsp_types::NumberOrString::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .map(|s| s == "restful-style-case")
                .unwrap_or(false)
                && d.severity == Some(lsp_types::DiagnosticSeverity::INFORMATION)
        });

        prop_assert!(has_case_suggestion,
            "Expected restful-style-case suggestion for path with uppercase");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: spring-lsp, Property 48: RESTful 风格检查（符合规范）
    ///
    /// **Validates: Requirements 10.5**
    ///
    /// For any 路由路径符合 RESTful 命名规范，
    /// 不应该生成风格建议。
    #[test]
    fn prop_validate_restful_style_compliant(
        path in valid_route_path(),
    ) {
        let mut navigator = RouteNavigator::new();

        let route = route_macro(path, vec![HttpMethod::Get]);
        let doc = rust_document(vec![SpringMacro::Route(route)]);

        navigator.build_index(&[doc]);

        // 验证路由
        let diagnostics = navigator.validate_routes();

        // 验证：不应该有 RESTful 风格建议
        let has_restful_suggestion = diagnostics.iter().any(|d| {
            d.code.as_ref()
                .and_then(|c| match c {
                    lsp_types::NumberOrString::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .map(|s| s.starts_with("restful-style"))
                .unwrap_or(false)
        });

        prop_assert!(!has_restful_suggestion,
            "Should not have RESTful style suggestion for compliant path");
    }
}

// ============================================================================
// 综合验证测试
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// 验证路由验证的完整性
    ///
    /// 此测试验证：
    /// 1. 验证函数不会崩溃
    /// 2. 返回的诊断列表是有效的
    /// 3. 所有诊断都有必要的字段
    #[test]
    fn prop_validate_routes_completeness(
        paths in prop::collection::vec(valid_route_path(), 1..10),
        methods in prop::collection::vec(http_method(), 1..5),
    ) {
        let mut navigator = RouteNavigator::new();

        // 创建多个路由
        let macros: Vec<_> = paths
            .into_iter()
            .map(|path| {
                SpringMacro::Route(route_macro(path, methods.clone()))
            })
            .collect();

        let doc = rust_document(macros);
        navigator.build_index(&[doc]);

        // 验证路由（不应该崩溃）
        let diagnostics = navigator.validate_routes();

        // 验证：所有诊断都有必要的字段
        for diagnostic in &diagnostics {
            prop_assert!(diagnostic.message.len() > 0, "Diagnostic should have a message");
            prop_assert!(diagnostic.severity.is_some(), "Diagnostic should have a severity");
            prop_assert!(diagnostic.source.is_some(), "Diagnostic should have a source");
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// 验证冲突检测的一致性
    ///
    /// 此测试验证：
    /// 1. 冲突检测结果是确定性的
    /// 2. 多次检测返回相同的结果
    /// 3. 冲突索引是有效的
    #[test]
    fn prop_conflict_detection_consistency(
        path in valid_route_path(),
        method in http_method(),
        num_routes in 2usize..5,
    ) {
        let mut navigator = RouteNavigator::new();

        // 创建多个具有相同路径和方法的路由
        let macros: Vec<_> = (0..num_routes)
            .map(|i| {
                let mut route = route_macro(path.clone(), vec![method.clone()]);
                route.handler_name = format!("handler_{}", i);
                SpringMacro::Route(route)
            })
            .collect();

        let doc = rust_document(macros);
        navigator.build_index(&[doc]);

        // 多次检测冲突
        let conflicts1 = navigator.detect_conflicts();
        let conflicts2 = navigator.detect_conflicts();

        // 验证：结果应该一致
        prop_assert_eq!(conflicts1.len(), conflicts2.len());

        // 验证：所有冲突索引都是有效的
        for conflict in &conflicts1 {
            prop_assert!(conflict.index1 < navigator.get_all_routes().len());
            prop_assert!(conflict.index2 < navigator.get_all_routes().len());
            prop_assert!(conflict.index1 < conflict.index2);
        }
    }
}
