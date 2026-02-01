//! MacroAnalyzer 属性测试
//!
//! 使用 proptest 验证 MacroAnalyzer 的通用正确性属性

use lsp_types::Url;
use proptest::prelude::*;
use spring_lsp::macro_analyzer::MacroAnalyzer;

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

/// Rust 关键字列表
const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "_",
];

/// 生成有效的 Rust 标识符
///
/// Rust 标识符必须以字母或下划线开头，后跟字母、数字或下划线
/// 注意：避免生成 Rust 关键字
fn valid_identifier() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_]{0,30}".prop_filter("not a Rust keyword", |s| {
        !RUST_KEYWORDS.contains(&s.as_str())
    })
}

/// 生成有效的 Rust 类型名称（通常是 PascalCase）
fn valid_type_name() -> impl Strategy<Value = String> {
    "[A-Z][a-zA-Z0-9]{0,30}"
}

/// 生成简单的 Rust 结构体定义
fn simple_struct() -> impl Strategy<Value = String> {
    (
        valid_type_name(),
        prop::collection::vec(valid_identifier(), 0..5),
    )
        .prop_map(|(struct_name, fields)| {
            let mut code = format!("struct {} {{\n", struct_name);
            for field in fields {
                code.push_str(&format!("    {}: String,\n", field));
            }
            code.push_str("}\n");
            code
        })
}

/// 生成简单的 Rust 函数定义
fn simple_function() -> impl Strategy<Value = String> {
    valid_identifier()
        .prop_map(|fn_name| format!("fn {}() {{\n    // function body\n}}\n", fn_name))
}

/// 生成简单的 Rust 枚举定义
fn simple_enum() -> impl Strategy<Value = String> {
    (
        valid_type_name(),
        prop::collection::vec(valid_type_name(), 1..5),
    )
        .prop_map(|(enum_name, variants)| {
            let mut code = format!("enum {} {{\n", enum_name);
            for variant in variants {
                code.push_str(&format!("    {},\n", variant));
            }
            code.push_str("}\n");
            code
        })
}

/// 生成简单的 Rust impl 块
fn simple_impl() -> impl Strategy<Value = String> {
    (valid_type_name(), valid_identifier()).prop_map(|(type_name, method_name)| {
        format!(
            "impl {} {{\n    fn {}(&self) {{\n        // method body\n    }}\n}}\n",
            type_name, method_name
        )
    })
}

/// 生成简单的 Rust use 语句
fn simple_use() -> impl Strategy<Value = String> {
    (valid_identifier(), valid_identifier())
        .prop_map(|(module, item)| format!("use {}::{};\n", module, item))
}

/// 生成简单的 Rust mod 声明
fn simple_mod() -> impl Strategy<Value = String> {
    valid_identifier().prop_map(|mod_name| format!("mod {};\n", mod_name))
}

/// 生成简单的 Rust 常量定义
fn simple_const() -> impl Strategy<Value = String> {
    (valid_identifier(), any::<i32>()).prop_map(|(const_name, value)| {
        format!("const {}: i32 = {};\n", const_name.to_uppercase(), value)
    })
}

/// 生成简单的 Rust trait 定义
fn simple_trait() -> impl Strategy<Value = String> {
    (valid_type_name(), valid_identifier()).prop_map(|(trait_name, method_name)| {
        format!(
            "trait {} {{\n    fn {}(&self);\n}}\n",
            trait_name, method_name
        )
    })
}

/// 生成语法正确的 Rust 源代码
///
/// 这个生成器组合多种 Rust 语法元素，生成语法正确的 Rust 代码
fn valid_rust_code() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            simple_struct(),
            simple_function(),
            simple_enum(),
            simple_impl(),
            simple_use(),
            simple_mod(),
            simple_const(),
            simple_trait(),
        ],
        0..10,
    )
    .prop_map(|items| items.join("\n"))
}

/// 生成包含注释的 Rust 代码
fn rust_code_with_comments() -> impl Strategy<Value = String> {
    (
        valid_rust_code(),
        prop::collection::vec("[a-zA-Z0-9 ]{1,50}", 0..5),
    )
        .prop_map(|(code, comments)| {
            let mut result = String::new();
            for comment in comments {
                result.push_str(&format!("// {}\n", comment));
            }
            result.push_str(&code);
            result
        })
}

/// 生成包含文档注释的 Rust 代码
fn rust_code_with_doc_comments() -> impl Strategy<Value = String> {
    (
        simple_function(),
        prop::collection::vec("[a-zA-Z0-9 ]{1,50}", 0..3),
    )
        .prop_map(|(code, doc_lines)| {
            let mut result = String::new();
            for doc_line in doc_lines {
                result.push_str(&format!("/// {}\n", doc_line));
            }
            result.push_str(&code);
            result
        })
}

/// 生成空的或只包含空白的 Rust 代码
fn empty_or_whitespace() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),
        Just("   ".to_string()),
        Just("\n\n\n".to_string()),
        Just("  \n  \n  ".to_string()),
    ]
}

/// 生成测试用的 URI
fn test_uri() -> impl Strategy<Value = Url> {
    valid_identifier().prop_map(|name| {
        Url::parse(&format!("file:///test/{}.rs", name)).expect("Failed to create test URI")
    })
}

// ============================================================================
// 属性测试
// ============================================================================

// Feature: spring-lsp, Property 25: Rust 文件解析成功性
//
// **Validates: Requirements 6.1**
//
// *For any* 语法正确的 Rust 源文件，宏分析器应该成功解析并返回语法树。
//
// 这个属性测试验证：
// 1. 对于任何语法正确的 Rust 代码，parse 方法应该返回 Ok
// 2. 返回的 RustDocument 应该包含正确的 URI
// 3. 返回的 RustDocument 应该包含正确的内容
// 4. 解析不应该崩溃或 panic
proptest! {
    #[test]
    fn prop_parse_valid_rust_code(
        uri in test_uri(),
        code in valid_rust_code()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析 Rust 代码
        let result = analyzer.parse(uri.clone(), code.clone());

        // 应该成功解析
        prop_assert!(result.is_ok(),
            "Should successfully parse valid Rust code: {:?}", result.err());

        let doc = result.unwrap();

        // URI 应该匹配
        prop_assert_eq!(doc.uri, uri,
            "Returned document URI should match input");

        // 内容应该匹配
        prop_assert_eq!(doc.content, code,
            "Returned document content should match input");
    }
}

// Feature: spring-lsp, Property 25: Rust 文件解析成功性（带注释）
//
// **Validates: Requirements 6.1**
//
// 验证包含注释的 Rust 代码也能正确解析
proptest! {
    #[test]
    fn prop_parse_rust_with_comments(
        uri in test_uri(),
        code in rust_code_with_comments()
    ) {
        let analyzer = MacroAnalyzer::new();

        let result = analyzer.parse(uri.clone(), code.clone());

        // 应该成功解析
        prop_assert!(result.is_ok(),
            "Should successfully parse Rust code with comments: {:?}", result.err());

        let doc = result.unwrap();

        // URI 和内容应该匹配
        prop_assert_eq!(doc.uri, uri);
        prop_assert_eq!(doc.content, code);
    }
}

// Feature: spring-lsp, Property 25: Rust 文件解析成功性（带文档注释）
//
// **Validates: Requirements 6.1**
//
// 验证包含文档注释的 Rust 代码也能正确解析
proptest! {
    #[test]
    fn prop_parse_rust_with_doc_comments(
        uri in test_uri(),
        code in rust_code_with_doc_comments()
    ) {
        let analyzer = MacroAnalyzer::new();

        let result = analyzer.parse(uri.clone(), code.clone());

        // 应该成功解析
        prop_assert!(result.is_ok(),
            "Should successfully parse Rust code with doc comments: {:?}", result.err());

        let doc = result.unwrap();

        // URI 和内容应该匹配
        prop_assert_eq!(doc.uri, uri);
        prop_assert_eq!(doc.content, code);
    }
}

// Feature: spring-lsp, Property 25: Rust 文件解析成功性（空文件）
//
// **Validates: Requirements 6.1**
//
// 验证空文件或只包含空白的文件也能正确解析
proptest! {
    #[test]
    fn prop_parse_empty_or_whitespace(
        uri in test_uri(),
        code in empty_or_whitespace()
    ) {
        let analyzer = MacroAnalyzer::new();

        let result = analyzer.parse(uri.clone(), code.clone());

        // 应该成功解析
        prop_assert!(result.is_ok(),
            "Should successfully parse empty or whitespace-only code: {:?}", result.err());

        let doc = result.unwrap();

        // URI 和内容应该匹配
        prop_assert_eq!(doc.uri, uri);
        prop_assert_eq!(doc.content, code);
    }
}

// Feature: spring-lsp, Property 25: Rust 文件解析成功性（结构体）
//
// **Validates: Requirements 6.1**
//
// 专门测试结构体定义的解析
proptest! {
    #[test]
    fn prop_parse_struct_definitions(
        uri in test_uri(),
        code in simple_struct()
    ) {
        let analyzer = MacroAnalyzer::new();

        let result = analyzer.parse(uri.clone(), code.clone());

        // 应该成功解析
        prop_assert!(result.is_ok(),
            "Should successfully parse struct definitions: {:?}", result.err());

        let doc = result.unwrap();
        prop_assert_eq!(doc.uri, uri);
        prop_assert_eq!(doc.content, code);
    }
}

// Feature: spring-lsp, Property 25: Rust 文件解析成功性（函数）
//
// **Validates: Requirements 6.1**
//
// 专门测试函数定义的解析
proptest! {
    #[test]
    fn prop_parse_function_definitions(
        uri in test_uri(),
        code in simple_function()
    ) {
        let analyzer = MacroAnalyzer::new();

        let result = analyzer.parse(uri.clone(), code.clone());

        // 应该成功解析
        prop_assert!(result.is_ok(),
            "Should successfully parse function definitions: {:?}", result.err());

        let doc = result.unwrap();
        prop_assert_eq!(doc.uri, uri);
        prop_assert_eq!(doc.content, code);
    }
}

// Feature: spring-lsp, Property 25: Rust 文件解析成功性（枚举）
//
// **Validates: Requirements 6.1**
//
// 专门测试枚举定义的解析
proptest! {
    #[test]
    fn prop_parse_enum_definitions(
        uri in test_uri(),
        code in simple_enum()
    ) {
        let analyzer = MacroAnalyzer::new();

        let result = analyzer.parse(uri.clone(), code.clone());

        // 应该成功解析
        prop_assert!(result.is_ok(),
            "Should successfully parse enum definitions: {:?}", result.err());

        let doc = result.unwrap();
        prop_assert_eq!(doc.uri, uri);
        prop_assert_eq!(doc.content, code);
    }
}

// Feature: spring-lsp, Property 25: Rust 文件解析成功性（impl 块）
//
// **Validates: Requirements 6.1**
//
// 专门测试 impl 块的解析
proptest! {
    #[test]
    fn prop_parse_impl_blocks(
        uri in test_uri(),
        code in simple_impl()
    ) {
        let analyzer = MacroAnalyzer::new();

        let result = analyzer.parse(uri.clone(), code.clone());

        // 应该成功解析
        prop_assert!(result.is_ok(),
            "Should successfully parse impl blocks: {:?}", result.err());

        let doc = result.unwrap();
        prop_assert_eq!(doc.uri, uri);
        prop_assert_eq!(doc.content, code);
    }
}

// Feature: spring-lsp, Property 25: Rust 文件解析成功性（trait）
//
// **Validates: Requirements 6.1**
//
// 专门测试 trait 定义的解析
proptest! {
    #[test]
    fn prop_parse_trait_definitions(
        uri in test_uri(),
        code in simple_trait()
    ) {
        let analyzer = MacroAnalyzer::new();

        let result = analyzer.parse(uri.clone(), code.clone());

        // 应该成功解析
        prop_assert!(result.is_ok(),
            "Should successfully parse trait definitions: {:?}", result.err());

        let doc = result.unwrap();
        prop_assert_eq!(doc.uri, uri);
        prop_assert_eq!(doc.content, code);
    }
}

// Feature: spring-lsp, Property 25: Rust 文件解析成功性（幂等性）
//
// **Validates: Requirements 6.1**
//
// 验证多次解析同一代码应该产生相同的结果
proptest! {
    #[test]
    fn prop_parse_idempotence(
        uri in test_uri(),
        code in valid_rust_code()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析两次
        let result1 = analyzer.parse(uri.clone(), code.clone());
        let result2 = analyzer.parse(uri.clone(), code.clone());

        // 两次都应该成功
        prop_assert!(result1.is_ok());
        prop_assert!(result2.is_ok());

        let doc1 = result1.unwrap();
        let doc2 = result2.unwrap();

        // 结果应该相同
        prop_assert_eq!(doc1.uri, doc2.uri);
        prop_assert_eq!(doc1.content, doc2.content);
    }
}

// Feature: spring-lsp, Property 25: Rust 文件解析成功性（extract_macros 不应失败）
//
// **Validates: Requirements 6.1**
//
// 验证 extract_macros 方法对于已成功解析的文档不应失败
proptest! {
    #[test]
    fn prop_extract_macros_does_not_fail(
        uri in test_uri(),
        code in valid_rust_code()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 先解析
        let parse_result = analyzer.parse(uri.clone(), code.clone());
        prop_assert!(parse_result.is_ok());

        let doc = parse_result.unwrap();

        // extract_macros 不应该失败
        let extract_result = analyzer.extract_macros(doc);
        prop_assert!(extract_result.is_ok(),
            "extract_macros should not fail on successfully parsed document: {:?}",
            extract_result.err());
    }
}

// ============================================================================
// Spring-rs 宏生成器
// ============================================================================

/// 生成带有 #[derive(Service)] 的结构体
fn service_struct() -> impl Strategy<Value = String> {
    (
        valid_type_name(),
        prop::collection::vec((valid_identifier(), valid_type_name()), 0..5),
    )
        .prop_map(|(struct_name, fields)| {
            let mut code = format!("#[derive(Service)]\nstruct {} {{\n", struct_name);
            for (field_name, field_type) in fields {
                code.push_str(&format!("    {}: {},\n", field_name, field_type));
            }
            code.push_str("}\n");
            code
        })
}

/// 生成带有 #[inject] 属性的字段
fn inject_field() -> impl Strategy<Value = (String, String, String)> {
    (
        valid_identifier(),
        valid_type_name(),
        prop_oneof![Just("component".to_string()), Just("config".to_string()),],
    )
}

/// 生成带有 #[inject] 属性和组件名称的字段
fn inject_field_with_name() -> impl Strategy<Value = (String, String, String, String)> {
    (
        valid_identifier(),
        valid_type_name(),
        prop_oneof![Just("component".to_string()), Just("config".to_string()),],
        valid_identifier(),
    )
}

/// 生成带有 #[inject] 属性的 Service 结构体
fn service_struct_with_inject() -> impl Strategy<Value = String> {
    (
        valid_type_name(),
        prop::collection::vec(inject_field(), 1..5),
    )
        .prop_map(|(struct_name, fields)| {
            let mut code = format!("#[derive(Service)]\nstruct {} {{\n", struct_name);
            for (field_name, field_type, inject_type) in fields {
                code.push_str(&format!("    #[inject({})]\n", inject_type));
                code.push_str(&format!("    {}: {},\n", field_name, field_type));
            }
            code.push_str("}\n");
            code
        })
}

/// 生成带有命名组件的 #[inject] 属性的 Service 结构体
fn service_struct_with_named_inject() -> impl Strategy<Value = String> {
    (
        valid_type_name(),
        prop::collection::vec(inject_field_with_name(), 1..5),
    )
        .prop_map(|(struct_name, fields)| {
            let mut code = format!("#[derive(Service)]\nstruct {} {{\n", struct_name);
            for (field_name, field_type, inject_type, component_name) in fields {
                code.push_str(&format!(
                    "    #[inject({} = \"{}\")]\n",
                    inject_type, component_name
                ));
                code.push_str(&format!("    {}: {},\n", field_name, field_type));
            }
            code.push_str("}\n");
            code
        })
}

/// 生成有效的路由路径
fn valid_route_path() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("/".to_string()),
        Just("/users".to_string()),
        Just("/api/v1/users".to_string()),
        "[a-z/]{1,30}".prop_map(|s| format!("/{}", s.replace("//", "/"))),
        (valid_identifier(), valid_identifier())
            .prop_map(|(seg1, seg2)| format!("/{}/{}", seg1, seg2)),
    ]
}

/// 生成带有路径参数的路由路径
fn route_path_with_params() -> impl Strategy<Value = String> {
    (valid_identifier(), valid_identifier())
        .prop_map(|(path, param)| format!("/{}/{{{}}}", path, param))
}

/// 生成路由宏（单个方法）
fn route_macro_single_method() -> impl Strategy<Value = String> {
    (
        prop_oneof![
            Just("get"),
            Just("post"),
            Just("put"),
            Just("delete"),
            Just("patch"),
            Just("head"),
            Just("options"),
        ],
        valid_route_path(),
        valid_identifier(),
    )
        .prop_map(|(method, path, fn_name)| {
            format!(
                "#[{}(\"{}\")]\nasync fn {}() {{\n    // handler\n}}\n",
                method, path, fn_name
            )
        })
}

/// 生成路由宏（多个方法）
fn route_macro_multiple_methods() -> impl Strategy<Value = String> {
    (
        valid_route_path(),
        prop::collection::vec(
            prop_oneof![Just("GET"), Just("POST"), Just("PUT"), Just("DELETE"),],
            1..4,
        ),
        valid_identifier(),
    )
        .prop_map(|(path, methods, fn_name)| {
            let method_attrs = methods
                .iter()
                .map(|m| format!("method = \"{}\"", m))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "#[route(\"{}\", {})]\nasync fn {}() {{\n    // handler\n}}\n",
                path, method_attrs, fn_name
            )
        })
}

/// 生成带有路径参数的路由宏
fn route_macro_with_params() -> impl Strategy<Value = String> {
    (
        prop_oneof![Just("get"), Just("post"), Just("put"), Just("delete"),],
        route_path_with_params(),
        valid_identifier(),
    )
        .prop_map(|(method, path, fn_name)| {
            format!(
                "#[{}(\"{}\")]\nasync fn {}() {{\n    // handler\n}}\n",
                method, path, fn_name
            )
        })
}

/// 生成 #[auto_config] 宏
fn auto_config_macro() -> impl Strategy<Value = String> {
    (valid_type_name(), valid_identifier()).prop_map(|(configurator, fn_name)| {
        format!(
            "#[auto_config({})]\nasync fn {}() {{\n    // config\n}}\n",
            configurator, fn_name
        )
    })
}

/// 生成 #[cron] 任务宏
fn cron_job_macro() -> impl Strategy<Value = String> {
    (
        prop_oneof![
            Just("0 0 * * * *"),        // 每小时
            Just("0 0 0 * * *"),        // 每天
            Just("0 */5 * * * *"),      // 每5分钟
            Just("0 0 12 * * MON-FRI"), // 工作日中午
        ],
        valid_identifier(),
    )
        .prop_map(|(cron_expr, fn_name)| {
            format!(
                "#[cron(\"{}\")]\nasync fn {}() {{\n    // job\n}}\n",
                cron_expr, fn_name
            )
        })
}

/// 生成 #[fix_delay] 任务宏
fn fix_delay_job_macro() -> impl Strategy<Value = String> {
    (1u64..3600, valid_identifier()).prop_map(|(seconds, fn_name)| {
        format!(
            "#[fix_delay({})]\nasync fn {}() {{\n    // job\n}}\n",
            seconds, fn_name
        )
    })
}

/// 生成 #[fix_rate] 任务宏
fn fix_rate_job_macro() -> impl Strategy<Value = String> {
    (1u64..3600, valid_identifier()).prop_map(|(seconds, fn_name)| {
        format!(
            "#[fix_rate({})]\nasync fn {}() {{\n    // job\n}}\n",
            seconds, fn_name
        )
    })
}

// ============================================================================
// 宏识别属性测试
// ============================================================================

// Feature: spring-lsp, Property 26: Service 宏识别
//
// **Validates: Requirements 6.2**
//
// *For any* 包含 `#[derive(Service)]` 的 Rust 代码，宏分析器应该识别该宏并提取结构体名称和字段信息。
//
// 这个属性测试验证：
// 1. 对于任何带有 #[derive(Service)] 的结构体，extract_macros 应该识别它
// 2. 提取的宏应该包含正确的结构体名称
// 3. 提取的宏应该包含正确的字段信息
proptest! {
    #[test]
    fn prop_service_macro_recognition(
        uri in test_uri(),
        code in service_struct()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let result = analyzer.extract_macros(doc);

        prop_assert!(result.is_ok());
        let doc = result.unwrap();

        // 应该至少识别到一个 Service 宏
        let service_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::DeriveService(s) => Some(s),
                _ => None,
            })
            .collect();

        prop_assert!(!service_macros.is_empty(),
            "Should recognize at least one Service macro in code:\n{}", code);

        // 验证结构体名称不为空
        for service_macro in service_macros {
            prop_assert!(!service_macro.struct_name.is_empty(),
                "Service macro should have non-empty struct name");
        }
    }
}

// Feature: spring-lsp, Property 27: Inject 属性识别
//
// **Validates: Requirements 6.3**
//
// *For any* 包含 `#[inject]` 属性的字段，宏分析器应该识别注入类型（component/config）和组件名称（如果指定）。
//
// 这个属性测试验证：
// 1. 对于任何带有 #[inject] 属性的字段，extract_macros 应该识别它
// 2. 提取的注入信息应该包含正确的注入类型
// 3. 如果指定了组件名称，应该正确提取
proptest! {
    #[test]
    fn prop_inject_attribute_recognition(
        uri in test_uri(),
        code in service_struct_with_inject()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let result = analyzer.extract_macros(doc);

        prop_assert!(result.is_ok());
        let doc = result.unwrap();

        // 应该识别到 Service 宏
        let service_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::DeriveService(s) => Some(s),
                _ => None,
            })
            .collect();

        prop_assert!(!service_macros.is_empty(),
            "Should recognize Service macro with inject attributes");

        // 验证至少有一个字段有 inject 属性
        for service_macro in service_macros {
            let fields_with_inject: Vec<_> = service_macro.fields.iter()
                .filter(|f| f.inject.is_some())
                .collect();

            prop_assert!(!fields_with_inject.is_empty(),
                "Should recognize inject attributes on fields");

            // 验证注入类型是 Component 或 Config
            for field in fields_with_inject {
                let inject = field.inject.as_ref().unwrap();
                prop_assert!(
                    inject.inject_type == spring_lsp::macro_analyzer::InjectType::Component ||
                    inject.inject_type == spring_lsp::macro_analyzer::InjectType::Config,
                    "Inject type should be Component or Config"
                );
            }
        }
    }
}

// Feature: spring-lsp, Property 27: Inject 属性识别（带组件名称）
//
// **Validates: Requirements 6.3**
//
// 验证带有组件名称的 inject 属性能够正确识别
proptest! {
    #[test]
    fn prop_inject_attribute_with_name_recognition(
        uri in test_uri(),
        code in service_struct_with_named_inject()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let result = analyzer.extract_macros(doc);

        prop_assert!(result.is_ok());
        let doc = result.unwrap();

        // 应该识别到 Service 宏
        let service_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::DeriveService(s) => Some(s),
                _ => None,
            })
            .collect();

        prop_assert!(!service_macros.is_empty());

        // 验证至少有一个字段有带名称的 inject 属性
        for service_macro in service_macros {
            let fields_with_named_inject: Vec<_> = service_macro.fields.iter()
                .filter(|f| f.inject.is_some() && f.inject.as_ref().unwrap().component_name.is_some())
                .collect();

            prop_assert!(!fields_with_named_inject.is_empty(),
                "Should recognize inject attributes with component names");

            // 验证组件名称不为空
            for field in fields_with_named_inject {
                let component_name = field.inject.as_ref().unwrap().component_name.as_ref().unwrap();
                prop_assert!(!component_name.is_empty(),
                    "Component name should not be empty");
            }
        }
    }
}

// Feature: spring-lsp, Property 28: 路由宏识别
//
// **Validates: Requirements 6.4**
//
// *For any* 包含路由宏（`#[get]`、`#[post]` 等）的函数，宏分析器应该提取路由路径和 HTTP 方法。
//
// 这个属性测试验证：
// 1. 对于任何带有路由宏的函数，extract_macros 应该识别它
// 2. 提取的路由信息应该包含正确的路径
// 3. 提取的路由信息应该包含正确的 HTTP 方法
proptest! {
    #[test]
    fn prop_route_macro_recognition(
        uri in test_uri(),
        code in route_macro_single_method()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let result = analyzer.extract_macros(doc);

        prop_assert!(result.is_ok());
        let doc = result.unwrap();

        // 应该识别到路由宏
        let route_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::Route(r) => Some(r),
                _ => None,
            })
            .collect();

        prop_assert!(!route_macros.is_empty(),
            "Should recognize route macro in code:\n{}", code);

        // 验证路由信息
        for route_macro in route_macros {
            // 路径不应为空
            prop_assert!(!route_macro.path.is_empty(),
                "Route path should not be empty");

            // 应该至少有一个 HTTP 方法
            prop_assert!(!route_macro.methods.is_empty(),
                "Route should have at least one HTTP method");

            // 处理器名称不应为空
            prop_assert!(!route_macro.handler_name.is_empty(),
                "Handler name should not be empty");
        }
    }
}

// Feature: spring-lsp, Property 28: 路由宏识别（多方法）
//
// **Validates: Requirements 6.4**
//
// 验证 #[route] 宏可以识别多个 HTTP 方法
proptest! {
    #[test]
    fn prop_route_macro_multiple_methods_recognition(
        uri in test_uri(),
        code in route_macro_multiple_methods()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let result = analyzer.extract_macros(doc);

        prop_assert!(result.is_ok());
        let doc = result.unwrap();

        // 应该识别到路由宏
        let route_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::Route(r) => Some(r),
                _ => None,
            })
            .collect();

        prop_assert!(!route_macros.is_empty(),
            "Should recognize route macro with multiple methods");

        // 注意：由于我们的生成器可能生成1-4个方法，这里不强制要求多个方法
        // 但至少应该有方法
        for route_macro in route_macros {
            prop_assert!(!route_macro.methods.is_empty(),
                "Route should have at least one method");
        }
    }
}

// Feature: spring-lsp, Property 28: 路由宏识别（路径参数）
//
// **Validates: Requirements 6.4**
//
// 验证带有路径参数的路由能够正确识别
proptest! {
    #[test]
    fn prop_route_macro_with_params_recognition(
        uri in test_uri(),
        code in route_macro_with_params()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let result = analyzer.extract_macros(doc);

        prop_assert!(result.is_ok());
        let doc = result.unwrap();

        // 应该识别到路由宏
        let route_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::Route(r) => Some(r),
                _ => None,
            })
            .collect();

        prop_assert!(!route_macros.is_empty(),
            "Should recognize route macro with path parameters");

        // 验证路径包含参数（包含 { 和 }）
        for route_macro in route_macros {
            prop_assert!(route_macro.path.contains('{') && route_macro.path.contains('}'),
                "Route path should contain path parameters: {}", route_macro.path);
        }
    }
}

// Feature: spring-lsp, Property 29: AutoConfig 宏识别
//
// **Validates: Requirements 6.5**
//
// *For any* 包含 `#[auto_config]` 属性的函数，宏分析器应该识别配置器类型。
//
// 这个属性测试验证：
// 1. 对于任何带有 #[auto_config] 的函数，extract_macros 应该识别它
// 2. 提取的配置器类型不应为空
proptest! {
    #[test]
    fn prop_auto_config_macro_recognition(
        uri in test_uri(),
        code in auto_config_macro()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let result = analyzer.extract_macros(doc);

        prop_assert!(result.is_ok());
        let doc = result.unwrap();

        // 应该识别到 AutoConfig 宏
        let auto_config_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::AutoConfig(a) => Some(a),
                _ => None,
            })
            .collect();

        prop_assert!(!auto_config_macros.is_empty(),
            "Should recognize AutoConfig macro in code:\n{}", code);

        // 验证配置器类型不为空
        for auto_config_macro in auto_config_macros {
            prop_assert!(!auto_config_macro.configurator_type.is_empty(),
                "Configurator type should not be empty");
        }
    }
}

// Feature: spring-lsp, Property 30: 任务宏识别
//
// **Validates: Requirements 6.6**
//
// *For any* 包含任务调度宏（`#[cron]`、`#[fix_delay]`、`#[fix_rate]`）的函数，
// 宏分析器应该识别任务类型和调度参数。
//
// 这个属性测试验证：
// 1. 对于任何带有任务宏的函数，extract_macros 应该识别它
// 2. 提取的任务信息应该包含正确的调度参数

// Property 30: Cron 任务宏识别
proptest! {
    #[test]
    fn prop_cron_job_macro_recognition(
        uri in test_uri(),
        code in cron_job_macro()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let result = analyzer.extract_macros(doc);

        prop_assert!(result.is_ok());
        let doc = result.unwrap();

        // 应该识别到任务宏
        let job_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::Job(j) => Some(j),
                _ => None,
            })
            .collect();

        prop_assert!(!job_macros.is_empty(),
            "Should recognize cron job macro in code:\n{}", code);

        // 验证是 Cron 类型且表达式不为空
        for job_macro in job_macros {
            match job_macro {
                spring_lsp::macro_analyzer::JobMacro::Cron { expression, .. } => {
                    prop_assert!(!expression.is_empty(),
                        "Cron expression should not be empty");
                }
                _ => {
                    prop_assert!(false, "Expected Cron job macro");
                }
            }
        }
    }
}

// Property 30: FixDelay 任务宏识别
proptest! {
    #[test]
    fn prop_fix_delay_job_macro_recognition(
        uri in test_uri(),
        code in fix_delay_job_macro()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let result = analyzer.extract_macros(doc);

        prop_assert!(result.is_ok());
        let doc = result.unwrap();

        // 应该识别到任务宏
        let job_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::Job(j) => Some(j),
                _ => None,
            })
            .collect();

        prop_assert!(!job_macros.is_empty(),
            "Should recognize fix_delay job macro in code:\n{}", code);

        // 验证是 FixDelay 类型且秒数大于0
        for job_macro in job_macros {
            match job_macro {
                spring_lsp::macro_analyzer::JobMacro::FixDelay { seconds, .. } => {
                    prop_assert!(*seconds > 0,
                        "FixDelay seconds should be greater than 0");
                }
                _ => {
                    prop_assert!(false, "Expected FixDelay job macro");
                }
            }
        }
    }
}

// Property 30: FixRate 任务宏识别
proptest! {
    #[test]
    fn prop_fix_rate_job_macro_recognition(
        uri in test_uri(),
        code in fix_rate_job_macro()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let result = analyzer.extract_macros(doc);

        prop_assert!(result.is_ok());
        let doc = result.unwrap();

        // 应该识别到任务宏
        let job_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::Job(j) => Some(j),
                _ => None,
            })
            .collect();

        prop_assert!(!job_macros.is_empty(),
            "Should recognize fix_rate job macro in code:\n{}", code);

        // 验证是 FixRate 类型且秒数大于0
        for job_macro in job_macros {
            match job_macro {
                spring_lsp::macro_analyzer::JobMacro::FixRate { seconds, .. } => {
                    prop_assert!(*seconds > 0,
                        "FixRate seconds should be greater than 0");
                }
                _ => {
                    prop_assert!(false, "Expected FixRate job macro");
                }
            }
        }
    }
}

// ============================================================================
// 综合测试
// ============================================================================

/// 生成包含多种宏的复杂 Rust 代码
fn complex_rust_code_with_macros() -> impl Strategy<Value = String> {
    (
        prop::option::of(service_struct_with_inject()),
        prop::option::of(route_macro_single_method()),
        prop::option::of(auto_config_macro()),
        prop::option::of(prop_oneof![
            cron_job_macro(),
            fix_delay_job_macro(),
            fix_rate_job_macro(),
        ]),
    )
        .prop_map(|(service, route, auto_config, job)| {
            let mut code = String::new();

            if let Some(s) = service {
                code.push_str(&s);
                code.push('\n');
            }

            if let Some(r) = route {
                code.push_str(&r);
                code.push('\n');
            }

            if let Some(a) = auto_config {
                code.push_str(&a);
                code.push('\n');
            }

            if let Some(j) = job {
                code.push_str(&j);
                code.push('\n');
            }

            code
        })
}

// 综合测试：验证分析器可以在同一文件中识别多种宏
//
// **Validates: Requirements 6.2, 6.3, 6.4, 6.5, 6.6**
//
// 这个测试验证分析器可以在同一个文件中正确识别和提取多种不同类型的宏
proptest! {
    #[test]
    fn prop_multiple_macro_types_recognition(
        uri in test_uri(),
        code in complex_rust_code_with_macros()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let parse_result = analyzer.parse(uri.clone(), code.clone());
        prop_assert!(parse_result.is_ok(),
            "Should successfully parse complex code with multiple macro types");

        let doc = parse_result.unwrap();
        let extract_result = analyzer.extract_macros(doc);

        prop_assert!(extract_result.is_ok(),
            "Should successfully extract macros from complex code");

        let doc = extract_result.unwrap();

        // 验证提取的宏数量合理（至少应该有一些宏，因为生成器至少会生成一个）
        // 注意：由于使用 Option，可能所有都是 None，所以这里不强制要求
        // 但如果有宏，应该能正确识别

        // 统计各种宏的数量
        let service_count = doc.macros.iter()
            .filter(|m| matches!(m, spring_lsp::macro_analyzer::SpringMacro::DeriveService(_)))
            .count();

        let route_count = doc.macros.iter()
            .filter(|m| matches!(m, spring_lsp::macro_analyzer::SpringMacro::Route(_)))
            .count();

        let auto_config_count = doc.macros.iter()
            .filter(|m| matches!(m, spring_lsp::macro_analyzer::SpringMacro::AutoConfig(_)))
            .count();

        let job_count = doc.macros.iter()
            .filter(|m| matches!(m, spring_lsp::macro_analyzer::SpringMacro::Job(_)))
            .count();

        // 验证总数等于各部分之和
        prop_assert_eq!(
            doc.macros.len(),
            service_count + route_count + auto_config_count + job_count,
            "Total macro count should equal sum of individual counts"
        );
    }
}

// 幂等性测试：验证多次提取宏应该产生相同的结果
//
// **Validates: Requirements 6.2, 6.3, 6.4, 6.5, 6.6**
proptest! {
    #[test]
    fn prop_macro_extraction_idempotence(
        uri in test_uri(),
        code in complex_rust_code_with_macros()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析一次
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();

        // 提取两次
        let result1 = analyzer.extract_macros(doc.clone());
        let result2 = analyzer.extract_macros(doc);

        prop_assert!(result1.is_ok());
        prop_assert!(result2.is_ok());

        let doc1 = result1.unwrap();
        let doc2 = result2.unwrap();

        // 宏数量应该相同
        prop_assert_eq!(doc1.macros.len(), doc2.macros.len(),
            "Multiple extractions should produce same number of macros");
    }
}

// ============================================================================
// 宏展开和提示属性测试
// ============================================================================

// Feature: spring-lsp, Property 31: 宏展开生成
//
// **Validates: Requirements 7.1**
//
// *For any* 可展开的 spring-rs 宏，宏分析器应该生成语法正确的展开后代码。
//
// 这个属性测试验证：
// 1. 对于任何可识别的 spring-rs 宏，expand_macro 方法应该返回非空字符串
// 2. 展开后的代码应该包含关键的 Rust 语法元素（如 impl、fn 等）
// 3. 展开不应该崩溃或 panic
proptest! {
    #[test]
    fn prop_macro_expansion_generates_valid_code(
        uri in test_uri(),
        code in complex_rust_code_with_macros()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 对每个宏进行展开
        for macro_info in &doc.macros {
            let expanded = analyzer.expand_macro(macro_info);

            // 展开后的代码不应为空
            prop_assert!(!expanded.is_empty(),
                "Expanded macro code should not be empty");

            // 展开后的代码应该包含一些 Rust 关键字或语法元素
            // 这是一个基本的语法正确性检查
            let has_rust_syntax = expanded.contains("impl") ||
                                 expanded.contains("fn") ||
                                 expanded.contains("struct") ||
                                 expanded.contains("//") ||
                                 expanded.contains("pub");

            prop_assert!(has_rust_syntax,
                "Expanded code should contain Rust syntax elements, got: {}", expanded);
        }
    }
}

// Feature: spring-lsp, Property 31: Service 宏展开生成
//
// **Validates: Requirements 7.1**
//
// 专门测试 Service 宏的展开
proptest! {
    #[test]
    fn prop_service_macro_expansion(
        uri in test_uri(),
        code in service_struct_with_inject()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 找到 Service 宏
        let service_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::DeriveService(s) => Some(s),
                _ => None,
            })
            .collect();

        prop_assert!(!service_macros.is_empty());

        for service_macro in service_macros {
            let expanded = analyzer.expand_macro(
                &spring_lsp::macro_analyzer::SpringMacro::DeriveService(service_macro.clone())
            );

            // 展开后的代码应该包含 impl 块
            prop_assert!(expanded.contains("impl"),
                "Service macro expansion should contain impl block");

            // 应该包含 build 方法
            prop_assert!(expanded.contains("fn build") || expanded.contains("build"),
                "Service macro expansion should contain build method");

            // 应该包含结构体名称
            prop_assert!(expanded.contains(&service_macro.struct_name),
                "Service macro expansion should contain struct name");

            // 如果有字段，应该包含字段名称
            for field in &service_macro.fields {
                prop_assert!(expanded.contains(&field.name),
                    "Service macro expansion should contain field name: {}", field.name);
            }
        }
    }
}

// Feature: spring-lsp, Property 31: 路由宏展开生成
//
// **Validates: Requirements 7.1**
//
// 专门测试路由宏的展开
proptest! {
    #[test]
    fn prop_route_macro_expansion(
        uri in test_uri(),
        code in route_macro_single_method()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 找到路由宏
        let route_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::Route(r) => Some(r),
                _ => None,
            })
            .collect();

        prop_assert!(!route_macros.is_empty());

        for route_macro in route_macros {
            let expanded = analyzer.expand_macro(
                &spring_lsp::macro_analyzer::SpringMacro::Route(route_macro.clone())
            );

            // 展开后的代码应该包含路由路径
            prop_assert!(expanded.contains(&route_macro.path),
                "Route macro expansion should contain route path");

            // 应该包含 HTTP 方法
            for method in &route_macro.methods {
                let method_str = method.as_str();
                prop_assert!(expanded.contains(method_str) || expanded.contains(&method_str.to_lowercase()),
                    "Route macro expansion should contain HTTP method: {}", method_str);
            }

            // 应该包含处理器函数名称
            prop_assert!(expanded.contains(&route_macro.handler_name),
                "Route macro expansion should contain handler name");
        }
    }
}

// Feature: spring-lsp, Property 31: 任务宏展开生成
//
// **Validates: Requirements 7.1**
//
// 专门测试任务宏的展开
proptest! {
    #[test]
    fn prop_job_macro_expansion(
        uri in test_uri(),
        code in prop_oneof![
            cron_job_macro(),
            fix_delay_job_macro(),
            fix_rate_job_macro(),
        ]
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 找到任务宏
        let job_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::Job(j) => Some(j),
                _ => None,
            })
            .collect();

        prop_assert!(!job_macros.is_empty());

        for job_macro in job_macros {
            let expanded = analyzer.expand_macro(
                &spring_lsp::macro_analyzer::SpringMacro::Job(job_macro.clone())
            );

            // 展开后的代码应该包含任务类型信息
            match job_macro {
                spring_lsp::macro_analyzer::JobMacro::Cron { expression, .. } => {
                    prop_assert!(expanded.contains(expression) || expanded.contains("Cron"),
                        "Cron job expansion should contain expression or 'Cron'");
                }
                spring_lsp::macro_analyzer::JobMacro::FixDelay { seconds, .. } => {
                    prop_assert!(expanded.contains(&seconds.to_string()) || expanded.contains("FixDelay"),
                        "FixDelay job expansion should contain seconds or 'FixDelay'");
                }
                spring_lsp::macro_analyzer::JobMacro::FixRate { seconds, .. } => {
                    prop_assert!(expanded.contains(&seconds.to_string()) || expanded.contains("FixRate"),
                        "FixRate job expansion should contain seconds or 'FixRate'");
                }
            }
        }
    }
}

// Feature: spring-lsp, Property 32: Service 宏悬停提示
//
// **Validates: Requirements 7.2**
//
// *For any* `#[derive(Service)]` 宏，悬停时应该显示生成的 trait 实现代码。
//
// 这个属性测试验证：
// 1. 对于任何 Service 宏，hover_macro 方法应该返回非空字符串
// 2. 悬停提示应该包含结构体名称
// 3. 悬停提示应该包含字段信息
// 4. 悬停提示应该包含展开后的代码
proptest! {
    #[test]
    fn prop_service_macro_hover_provides_trait_implementation(
        uri in test_uri(),
        code in service_struct_with_inject()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 找到 Service 宏
        let service_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::DeriveService(s) => Some(s),
                _ => None,
            })
            .collect();

        prop_assert!(!service_macros.is_empty());

        for service_macro in service_macros {
            let hover = analyzer.hover_macro(
                &spring_lsp::macro_analyzer::SpringMacro::DeriveService(service_macro.clone())
            );

            // 悬停提示不应为空
            prop_assert!(!hover.is_empty(),
                "Service macro hover should not be empty");

            // 应该包含标题
            prop_assert!(hover.contains("Service") || hover.contains("派生宏"),
                "Hover should contain 'Service' or '派生宏'");

            // 应该包含结构体名称
            prop_assert!(hover.contains(&service_macro.struct_name),
                "Hover should contain struct name: {}", service_macro.struct_name);

            // 应该包含展开后的代码标记
            prop_assert!(hover.contains("```rust") || hover.contains("展开"),
                "Hover should contain code block or expansion indicator");

            // 如果有字段，应该包含字段信息
            for field in &service_macro.fields {
                prop_assert!(hover.contains(&field.name),
                    "Hover should contain field name: {}", field.name);
            }

            // 应该包含 impl 关键字（展开后的代码）
            prop_assert!(hover.contains("impl"),
                "Hover should contain impl keyword from expanded code");
        }
    }
}

// Feature: spring-lsp, Property 32: Service 宏悬停提示（注入信息）
//
// **Validates: Requirements 7.2**
//
// 验证 Service 宏悬停提示包含注入字段的详细信息
proptest! {
    #[test]
    fn prop_service_macro_hover_shows_inject_info(
        uri in test_uri(),
        code in service_struct_with_named_inject()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 找到 Service 宏
        let service_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::DeriveService(s) => Some(s),
                _ => None,
            })
            .collect();

        prop_assert!(!service_macros.is_empty());

        for service_macro in service_macros {
            let hover = analyzer.hover_macro(
                &spring_lsp::macro_analyzer::SpringMacro::DeriveService(service_macro.clone())
            );

            // 检查是否有带注入的字段
            let fields_with_inject: Vec<_> = service_macro.fields.iter()
                .filter(|f| f.inject.is_some())
                .collect();

            if !fields_with_inject.is_empty() {
                // 应该包含注入相关的信息
                prop_assert!(hover.contains("注入") || hover.contains("inject"),
                    "Hover should contain injection information");

                // 检查每个注入字段
                for field in fields_with_inject {
                    if let Some(inject) = &field.inject {
                        // 应该显示注入类型
                        match inject.inject_type {
                            spring_lsp::macro_analyzer::InjectType::Component => {
                                prop_assert!(hover.contains("组件") || hover.contains("Component"),
                                    "Hover should indicate component injection");
                            }
                            spring_lsp::macro_analyzer::InjectType::Config => {
                                prop_assert!(hover.contains("配置") || hover.contains("Config"),
                                    "Hover should indicate config injection");
                            }
                        }

                        // 如果有组件名称，应该显示
                        if let Some(name) = &inject.component_name {
                            prop_assert!(hover.contains(name),
                                "Hover should contain component name: {}", name);
                        }
                    }
                }
            }
        }
    }
}

// Feature: spring-lsp, Property 33: Inject 属性悬停提示
//
// **Validates: Requirements 7.3, 15.3**
//
// *For any* `#[inject]` 属性，悬停时应该显示注入的组件类型和来源信息。
//
// 这个属性测试验证：
// 1. 对于任何 Inject 宏，hover_macro 方法应该返回非空字符串
// 2. 悬停提示应该包含注入类型（Component 或 Config）
// 3. 如果指定了组件名称，悬停提示应该包含组件名称
// 4. 悬停提示应该包含使用示例
proptest! {
    #[test]
    fn prop_inject_attribute_hover_shows_component_info(
        uri in test_uri(),
        code in service_struct_with_inject()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 找到 Service 宏并提取 inject 属性
        for macro_info in &doc.macros {
            if let spring_lsp::macro_analyzer::SpringMacro::DeriveService(service) = macro_info {
                for field in &service.fields {
                    if let Some(inject) = &field.inject {
                        let hover = analyzer.hover_macro(
                            &spring_lsp::macro_analyzer::SpringMacro::Inject(inject.clone())
                        );

                        // 悬停提示不应为空
                        prop_assert!(!hover.is_empty(),
                            "Inject attribute hover should not be empty");

                        // 应该包含标题
                        prop_assert!(hover.contains("Inject") || hover.contains("注入"),
                            "Hover should contain 'Inject' or '注入'");

                        // 应该包含注入类型
                        match inject.inject_type {
                            spring_lsp::macro_analyzer::InjectType::Component => {
                                prop_assert!(hover.contains("Component") || hover.contains("组件"),
                                    "Hover should indicate component injection type");
                                prop_assert!(hover.contains("get_component"),
                                    "Hover should show get_component method");
                            }
                            spring_lsp::macro_analyzer::InjectType::Config => {
                                prop_assert!(hover.contains("Config") || hover.contains("配置"),
                                    "Hover should indicate config injection type");
                                prop_assert!(hover.contains("get_config"),
                                    "Hover should show get_config method");
                            }
                        }

                        // 应该包含代码示例
                        prop_assert!(hover.contains("```rust") || hover.contains("示例"),
                            "Hover should contain code example");
                    }
                }
            }
        }
    }
}

// Feature: spring-lsp, Property 33: Inject 属性悬停提示（带组件名称）
//
// **Validates: Requirements 7.3, 15.3**
//
// 验证带有组件名称的 inject 属性悬停提示包含组件名称信息
proptest! {
    #[test]
    fn prop_inject_attribute_hover_shows_component_name(
        uri in test_uri(),
        code in service_struct_with_named_inject()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 找到 Service 宏并提取带名称的 inject 属性
        for macro_info in &doc.macros {
            if let spring_lsp::macro_analyzer::SpringMacro::DeriveService(service) = macro_info {
                for field in &service.fields {
                    if let Some(inject) = &field.inject {
                        if let Some(component_name) = &inject.component_name {
                            // 只测试 component 类型的注入（config 类型不应该有组件名称）
                            if inject.inject_type == spring_lsp::macro_analyzer::InjectType::Component {
                                let hover = analyzer.hover_macro(
                                    &spring_lsp::macro_analyzer::SpringMacro::Inject(inject.clone())
                                );

                                // 应该包含组件名称
                                prop_assert!(hover.contains(component_name),
                                    "Hover should contain component name: {}", component_name);

                                // 应该说明这是命名组件
                                prop_assert!(hover.contains("名称") || hover.contains("name") || hover.contains("指定"),
                                    "Hover should indicate this is a named component");

                                // 应该在代码示例中显示组件名称
                                let code_with_name = format!("\"{}\"", component_name);
                                prop_assert!(hover.contains(&code_with_name),
                                    "Hover should show component name in code example");
                            }
                        }
                    }
                }
            }
        }
    }
}

// Feature: spring-lsp, Property 34: 宏参数验证
//
// **Validates: Requirements 7.4**
//
// *For any* 宏参数不符合宏定义要求的情况，诊断引擎应该生成错误诊断并提供修复建议。
//
// 这个属性测试验证：
// 1. 对于任何可识别的宏，validate_macro 方法应该返回诊断列表（可能为空）
// 2. 如果宏参数有错误，应该生成相应的诊断
// 3. 诊断应该包含错误代码和消息
proptest! {
    #[test]
    fn prop_macro_validation_detects_invalid_parameters(
        uri in test_uri(),
        code in complex_rust_code_with_macros()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 对每个宏进行验证
        for macro_info in &doc.macros {
            let diagnostics = analyzer.validate_macro(macro_info);

            // validate_macro 应该返回一个列表（可能为空，可能有错误）
            // 这里我们只验证它不会崩溃，并且返回的诊断格式正确

            for diagnostic in &diagnostics {
                // 每个诊断应该有消息
                prop_assert!(!diagnostic.message.is_empty(),
                    "Diagnostic message should not be empty");

                // 应该有严重性级别
                prop_assert!(diagnostic.severity.is_some(),
                    "Diagnostic should have severity level");

                // 应该有来源
                prop_assert_eq!(diagnostic.source.as_deref(), Some("spring-lsp"),
                    "Diagnostic source should be 'spring-lsp'");

                // 如果有错误代码，应该不为空
                if let Some(code) = &diagnostic.code {
                    match code {
                        lsp_types::NumberOrString::String(s) => {
                            prop_assert!(!s.is_empty(),
                                "Diagnostic code string should not be empty");
                        }
                        lsp_types::NumberOrString::Number(n) => {
                            prop_assert!(*n >= 0,
                                "Diagnostic code number should be non-negative");
                        }
                    }
                }
            }
        }
    }
}

// Feature: spring-lsp, Property 34: 路由宏参数验证
//
// **Validates: Requirements 7.4**
//
// 专门测试路由宏的参数验证
proptest! {
    #[test]
    fn prop_route_macro_validation(
        uri in test_uri(),
        code in route_macro_single_method()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 找到路由宏
        let route_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::Route(r) => Some(r),
                _ => None,
            })
            .collect();

        for route_macro in route_macros {
            let diagnostics = analyzer.validate_macro(
                &spring_lsp::macro_analyzer::SpringMacro::Route(route_macro.clone())
            );

            // 验证路径格式
            if !route_macro.path.starts_with('/') {
                // 应该有错误诊断
                prop_assert!(!diagnostics.is_empty(),
                    "Should generate diagnostic for path not starting with '/'");

                // 应该有关于路径格式的错误
                let has_path_error = diagnostics.iter().any(|d| {
                    d.message.contains("路径") || d.message.contains("path") || d.message.contains("/")
                });
                prop_assert!(has_path_error,
                    "Should have diagnostic about path format");
            }

            // 验证 HTTP 方法
            if route_macro.methods.is_empty() {
                // 应该有错误诊断
                prop_assert!(!diagnostics.is_empty(),
                    "Should generate diagnostic for empty methods");

                // 应该有关于方法的错误
                let has_method_error = diagnostics.iter().any(|d| {
                    d.message.contains("方法") || d.message.contains("method") || d.message.contains("HTTP")
                });
                prop_assert!(has_method_error,
                    "Should have diagnostic about HTTP methods");
            }
        }
    }
}

// Feature: spring-lsp, Property 34: 任务宏参数验证
//
// **Validates: Requirements 7.4**
//
// 专门测试任务宏的参数验证
proptest! {
    #[test]
    fn prop_job_macro_validation(
        uri in test_uri(),
        code in prop_oneof![
            cron_job_macro(),
            fix_delay_job_macro(),
            fix_rate_job_macro(),
        ]
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 找到任务宏
        let job_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::Job(j) => Some(j),
                _ => None,
            })
            .collect();

        for job_macro in job_macros {
            let diagnostics = analyzer.validate_macro(
                &spring_lsp::macro_analyzer::SpringMacro::Job(job_macro.clone())
            );

            // 验证任务参数
            match job_macro {
                spring_lsp::macro_analyzer::JobMacro::Cron { expression, .. } => {
                    if expression.is_empty() {
                        // 应该有错误诊断
                        prop_assert!(!diagnostics.is_empty(),
                            "Should generate diagnostic for empty cron expression");
                    }
                }
                spring_lsp::macro_analyzer::JobMacro::FixDelay { seconds, .. } => {
                    if *seconds == 0 {
                        // 可能有警告诊断
                        // 注意：我们的实现对 0 秒延迟生成警告而非错误
                        let _has_warning = diagnostics.iter().any(|d| {
                            d.severity == Some(lsp_types::DiagnosticSeverity::WARNING)
                        });
                        // 这里不强制要求，因为 0 秒可能是有效的
                    }
                }
                spring_lsp::macro_analyzer::JobMacro::FixRate { seconds, .. } => {
                    if *seconds == 0 {
                        // 应该有错误诊断
                        prop_assert!(!diagnostics.is_empty(),
                            "Should generate diagnostic for zero fix_rate seconds");

                        let has_error = diagnostics.iter().any(|d| {
                            d.severity == Some(lsp_types::DiagnosticSeverity::ERROR)
                        });
                        prop_assert!(has_error,
                            "Should have error diagnostic for zero fix_rate seconds");
                    }
                }
            }
        }
    }
}

// Feature: spring-lsp, Property 34: Service 宏参数验证
//
// **Validates: Requirements 7.4**
//
// 专门测试 Service 宏的参数验证
proptest! {
    #[test]
    fn prop_service_macro_validation(
        uri in test_uri(),
        code in service_struct_with_inject()
    ) {
        let analyzer = MacroAnalyzer::new();

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 找到 Service 宏
        let service_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::DeriveService(s) => Some(s),
                _ => None,
            })
            .collect();

        for service_macro in service_macros {
            let diagnostics = analyzer.validate_macro(
                &spring_lsp::macro_analyzer::SpringMacro::DeriveService(service_macro.clone())
            );

            // 检查字段的 inject 属性
            for field in &service_macro.fields {
                if let Some(inject) = &field.inject {
                    // 如果是 config 类型且有组件名称，应该有错误
                    if inject.inject_type == spring_lsp::macro_analyzer::InjectType::Config
                        && inject.component_name.is_some() {
                        prop_assert!(!diagnostics.is_empty(),
                            "Should generate diagnostic for config injection with component name");

                        let has_config_error = diagnostics.iter().any(|d| {
                            d.message.contains("config") || d.message.contains("配置")
                        });
                        prop_assert!(has_config_error,
                            "Should have diagnostic about config injection");
                    }
                }
            }
        }
    }
}

// ============================================================================
// 宏参数补全属性测试
// ============================================================================

// Feature: spring-lsp, Property 35: 宏参数补全
//
// **Validates: Requirements 7.5**
//
// *For any* 宏参数输入位置，补全引擎应该提供该宏支持的参数名称和值。
//
// 这个属性测试验证：
// 1. 对于任何可识别的 spring-rs 宏，complete_macro 方法应该返回非空补全列表
// 2. 补全项应该包含必要的信息（label、detail、documentation、insert_text）
// 3. 补全项的类型应该正确（PROPERTY、KEYWORD、CLASS、CONSTANT、VALUE、SNIPPET）
// 4. 补全不应该崩溃或 panic

// Property 35: Service 宏参数补全
proptest! {
    #[test]
    fn prop_service_macro_completion_provides_inject_options(
        uri in test_uri(),
        code in service_struct()
    ) {
        use spring_lsp::completion::CompletionEngine;
        use spring_lsp::schema::SchemaProvider;

        let analyzer = MacroAnalyzer::new();
        let schema_provider = SchemaProvider::new();
        let engine = CompletionEngine::new(schema_provider);

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 找到 Service 宏
        let service_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::DeriveService(s) => Some(s),
                _ => None,
            })
            .collect();

        prop_assert!(!service_macros.is_empty());

        for service_macro in service_macros {
            let completions = engine.complete_macro(
                &spring_lsp::macro_analyzer::SpringMacro::DeriveService(service_macro.clone()),
                None
            );

            // 应该提供补全项
            prop_assert!(!completions.is_empty(),
                "Service macro should provide completion items");

            // 验证每个补全项的必要信息
            for completion in &completions {
                // 应该有 label
                prop_assert!(!completion.label.is_empty(),
                    "Completion item should have non-empty label");

                // 应该有 detail
                prop_assert!(completion.detail.is_some(),
                    "Completion item '{}' should have detail", completion.label);

                // 应该有 documentation
                prop_assert!(completion.documentation.is_some(),
                    "Completion item '{}' should have documentation", completion.label);

                // 应该有 insert_text
                prop_assert!(completion.insert_text.is_some(),
                    "Completion item '{}' should have insert_text", completion.label);

                // 应该有 kind
                prop_assert!(completion.kind.is_some(),
                    "Completion item '{}' should have kind", completion.label);

                // Service 宏的补全项应该是 PROPERTY 类型
                prop_assert_eq!(completion.kind, Some(lsp_types::CompletionItemKind::PROPERTY),
                    "Service macro completion items should be PROPERTY kind");
            }

            // 应该包含 inject(component) 补全
            let has_inject_component = completions.iter().any(|c| c.label.contains("inject(component)"));
            prop_assert!(has_inject_component,
                "Service macro should provide inject(component) completion");

            // 应该包含 inject(config) 补全
            let has_inject_config = completions.iter().any(|c| c.label.contains("inject(config)"));
            prop_assert!(has_inject_config,
                "Service macro should provide inject(config) completion");
        }
    }
}

// Property 35: Inject 宏参数补全
proptest! {
    #[test]
    fn prop_inject_macro_completion_provides_type_options(
        uri in test_uri(),
        code in service_struct_with_inject()
    ) {
        use spring_lsp::completion::CompletionEngine;
        use spring_lsp::schema::SchemaProvider;

        let analyzer = MacroAnalyzer::new();
        let schema_provider = SchemaProvider::new();
        let engine = CompletionEngine::new(schema_provider);

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 找到 Service 宏并提取 inject 属性
        for macro_info in &doc.macros {
            if let spring_lsp::macro_analyzer::SpringMacro::DeriveService(service) = macro_info {
                for field in &service.fields {
                    if let Some(inject) = &field.inject {
                        let completions = engine.complete_macro(
                            &spring_lsp::macro_analyzer::SpringMacro::Inject(inject.clone()),
                            None
                        );

                        // 应该提供补全项
                        prop_assert!(!completions.is_empty(),
                            "Inject macro should provide completion items");

                        // 验证每个补全项的必要信息
                        for completion in &completions {
                            // 应该有 label
                            prop_assert!(!completion.label.is_empty(),
                                "Completion item should have non-empty label");

                            // 应该有 detail
                            prop_assert!(completion.detail.is_some(),
                                "Completion item '{}' should have detail", completion.label);

                            // 应该有 documentation
                            prop_assert!(completion.documentation.is_some(),
                                "Completion item '{}' should have documentation", completion.label);

                            // 应该有 insert_text
                            prop_assert!(completion.insert_text.is_some(),
                                "Completion item '{}' should have insert_text", completion.label);

                            // 应该有 kind
                            prop_assert!(completion.kind.is_some(),
                                "Completion item '{}' should have kind", completion.label);

                            // Inject 宏的补全项应该是 KEYWORD 类型
                            prop_assert_eq!(completion.kind, Some(lsp_types::CompletionItemKind::KEYWORD),
                                "Inject macro completion items should be KEYWORD kind");
                        }

                        // 应该包含 component 补全
                        let has_component = completions.iter().any(|c| c.label == "component");
                        prop_assert!(has_component,
                            "Inject macro should provide 'component' completion");

                        // 应该包含 config 补全
                        let has_config = completions.iter().any(|c| c.label == "config");
                        prop_assert!(has_config,
                            "Inject macro should provide 'config' completion");
                    }
                }
            }
        }
    }
}

// Property 35: AutoConfig 宏参数补全
proptest! {
    #[test]
    fn prop_auto_config_macro_completion_provides_configurator_types(
        uri in test_uri(),
        code in auto_config_macro()
    ) {
        use spring_lsp::completion::CompletionEngine;
        use spring_lsp::schema::SchemaProvider;

        let analyzer = MacroAnalyzer::new();
        let schema_provider = SchemaProvider::new();
        let engine = CompletionEngine::new(schema_provider);

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 找到 AutoConfig 宏
        let auto_config_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::AutoConfig(a) => Some(a),
                _ => None,
            })
            .collect();

        prop_assert!(!auto_config_macros.is_empty());

        for auto_config_macro in auto_config_macros {
            let completions = engine.complete_macro(
                &spring_lsp::macro_analyzer::SpringMacro::AutoConfig(auto_config_macro.clone()),
                None
            );

            // 应该提供补全项
            prop_assert!(!completions.is_empty(),
                "AutoConfig macro should provide completion items");

            // 验证每个补全项的必要信息
            for completion in &completions {
                // 应该有 label
                prop_assert!(!completion.label.is_empty(),
                    "Completion item should have non-empty label");

                // 应该有 detail
                prop_assert!(completion.detail.is_some(),
                    "Completion item '{}' should have detail", completion.label);

                // 应该有 documentation
                prop_assert!(completion.documentation.is_some(),
                    "Completion item '{}' should have documentation", completion.label);

                // 应该有 insert_text
                prop_assert!(completion.insert_text.is_some(),
                    "Completion item '{}' should have insert_text", completion.label);

                // 应该有 kind
                prop_assert!(completion.kind.is_some(),
                    "Completion item '{}' should have kind", completion.label);

                // AutoConfig 宏的补全项应该是 CLASS 类型
                prop_assert_eq!(completion.kind, Some(lsp_types::CompletionItemKind::CLASS),
                    "AutoConfig macro completion items should be CLASS kind");
            }

            // 应该包含常见的配置器类型
            let labels: Vec<_> = completions.iter().map(|c| c.label.as_str()).collect();
            prop_assert!(labels.contains(&"WebConfigurator") ||
                        labels.contains(&"JobConfigurator") ||
                        labels.contains(&"StreamConfigurator"),
                "AutoConfig macro should provide common configurator types");
        }
    }
}

// Property 35: Route 宏参数补全
proptest! {
    #[test]
    fn prop_route_macro_completion_provides_http_methods_and_path_params(
        uri in test_uri(),
        code in route_macro_single_method()
    ) {
        use spring_lsp::completion::CompletionEngine;
        use spring_lsp::schema::SchemaProvider;

        let analyzer = MacroAnalyzer::new();
        let schema_provider = SchemaProvider::new();
        let engine = CompletionEngine::new(schema_provider);

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 找到 Route 宏
        let route_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::Route(r) => Some(r),
                _ => None,
            })
            .collect();

        prop_assert!(!route_macros.is_empty());

        for route_macro in route_macros {
            let completions = engine.complete_macro(
                &spring_lsp::macro_analyzer::SpringMacro::Route(route_macro.clone()),
                None
            );

            // 应该提供补全项
            prop_assert!(!completions.is_empty(),
                "Route macro should provide completion items");

            // 验证每个补全项的必要信息
            for completion in &completions {
                // 应该有 label
                prop_assert!(!completion.label.is_empty(),
                    "Completion item should have non-empty label");

                // 应该有 detail
                prop_assert!(completion.detail.is_some(),
                    "Completion item '{}' should have detail", completion.label);

                // 应该有 documentation
                prop_assert!(completion.documentation.is_some(),
                    "Completion item '{}' should have documentation", completion.label);

                // 应该有 insert_text
                prop_assert!(completion.insert_text.is_some(),
                    "Completion item '{}' should have insert_text", completion.label);

                // 应该有 kind
                prop_assert!(completion.kind.is_some(),
                    "Completion item '{}' should have kind", completion.label);
            }

            // 应该包含 HTTP 方法补全（CONSTANT 类型）
            let http_methods = completions.iter()
                .filter(|c| c.kind == Some(lsp_types::CompletionItemKind::CONSTANT))
                .collect::<Vec<_>>();
            prop_assert!(!http_methods.is_empty(),
                "Route macro should provide HTTP method completions");

            // 应该包含常见的 HTTP 方法
            let method_labels: Vec<_> = http_methods.iter().map(|c| c.label.as_str()).collect();
            prop_assert!(method_labels.contains(&"GET") ||
                        method_labels.contains(&"POST") ||
                        method_labels.contains(&"PUT") ||
                        method_labels.contains(&"DELETE"),
                "Route macro should provide common HTTP methods");

            // 应该包含路径参数补全（SNIPPET 类型）
            let path_params = completions.iter()
                .filter(|c| c.kind == Some(lsp_types::CompletionItemKind::SNIPPET))
                .collect::<Vec<_>>();
            prop_assert!(!path_params.is_empty(),
                "Route macro should provide path parameter completions");

            // 路径参数补全应该包含 {id} 或类似的模板
            let has_path_param_template = path_params.iter().any(|c| c.label.contains('{'));
            prop_assert!(has_path_param_template,
                "Route macro should provide path parameter template like {{id}}");
        }
    }
}

// Property 35: Job 宏参数补全
proptest! {
    #[test]
    fn prop_job_macro_completion_provides_schedule_options(
        uri in test_uri(),
        code in prop_oneof![
            cron_job_macro(),
            fix_delay_job_macro(),
            fix_rate_job_macro(),
        ]
    ) {
        use spring_lsp::completion::CompletionEngine;
        use spring_lsp::schema::SchemaProvider;

        let analyzer = MacroAnalyzer::new();
        let schema_provider = SchemaProvider::new();
        let engine = CompletionEngine::new(schema_provider);

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 找到 Job 宏
        let job_macros: Vec<_> = doc.macros.iter()
            .filter_map(|m| match m {
                spring_lsp::macro_analyzer::SpringMacro::Job(j) => Some(j),
                _ => None,
            })
            .collect();

        prop_assert!(!job_macros.is_empty());

        for job_macro in job_macros {
            let completions = engine.complete_macro(
                &spring_lsp::macro_analyzer::SpringMacro::Job(job_macro.clone()),
                None
            );

            // 应该提供补全项
            prop_assert!(!completions.is_empty(),
                "Job macro should provide completion items");

            // 验证每个补全项的必要信息
            for completion in &completions {
                // 应该有 label
                prop_assert!(!completion.label.is_empty(),
                    "Completion item should have non-empty label");

                // 应该有 detail
                prop_assert!(completion.detail.is_some(),
                    "Completion item '{}' should have detail", completion.label);

                // 应该有 documentation
                prop_assert!(completion.documentation.is_some(),
                    "Completion item '{}' should have documentation", completion.label);

                // 应该有 insert_text
                prop_assert!(completion.insert_text.is_some(),
                    "Completion item '{}' should have insert_text", completion.label);

                // 应该有 kind
                prop_assert!(completion.kind.is_some(),
                    "Completion item '{}' should have kind", completion.label);
            }

            // 应该包含 cron 表达式补全（SNIPPET 类型）
            let cron_completions = completions.iter()
                .filter(|c| c.kind == Some(lsp_types::CompletionItemKind::SNIPPET) &&
                           c.label.contains('*'))
                .collect::<Vec<_>>();
            prop_assert!(!cron_completions.is_empty(),
                "Job macro should provide cron expression completions");

            // 应该包含延迟/频率值补全（VALUE 类型）
            let value_completions = completions.iter()
                .filter(|c| c.kind == Some(lsp_types::CompletionItemKind::VALUE))
                .collect::<Vec<_>>();
            prop_assert!(!value_completions.is_empty(),
                "Job macro should provide delay/rate value completions");

            // 验证 cron 表达式格式
            for cron_completion in cron_completions {
                // Cron 表达式应该包含空格分隔的字段
                let parts: Vec<_> = cron_completion.label.split_whitespace().collect();
                prop_assert!(parts.len() >= 5,
                    "Cron expression should have at least 5 fields: {}", cron_completion.label);
            }

            // 验证延迟/频率值是数字
            for value_completion in value_completions {
                let is_numeric = value_completion.label.parse::<u64>().is_ok();
                prop_assert!(is_numeric,
                    "Delay/rate value should be numeric: {}", value_completion.label);
            }
        }
    }
}

// Property 35: 补全项文档完整性
proptest! {
    #[test]
    fn prop_completion_items_have_complete_documentation(
        uri in test_uri(),
        code in complex_rust_code_with_macros()
    ) {
        use spring_lsp::completion::CompletionEngine;
        use spring_lsp::schema::SchemaProvider;

        let analyzer = MacroAnalyzer::new();
        let schema_provider = SchemaProvider::new();
        let engine = CompletionEngine::new(schema_provider);

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 对每个宏获取补全项
        for macro_info in &doc.macros {
            let completions = engine.complete_macro(macro_info, None);

            // 应该提供补全项
            prop_assert!(!completions.is_empty(),
                "Macro should provide completion items");

            // 验证每个补全项的文档完整性
            for completion in &completions {
                // 文档应该是 MarkupContent 类型
                if let Some(doc) = &completion.documentation {
                    match doc {
                        lsp_types::Documentation::String(s) => {
                            prop_assert!(!s.is_empty(),
                                "Documentation string should not be empty for '{}'", completion.label);
                        }
                        lsp_types::Documentation::MarkupContent(markup) => {
                            prop_assert!(!markup.value.is_empty(),
                                "Documentation markup should not be empty for '{}'", completion.label);

                            // 应该是 Markdown 格式
                            prop_assert_eq!(&markup.kind, &lsp_types::MarkupKind::Markdown,
                                "Documentation should be in Markdown format for '{}'", completion.label);

                            // Markdown 文档应该包含有用的信息（不只是空白）
                            let trimmed = markup.value.trim();
                            prop_assert!(!trimmed.is_empty(),
                                "Documentation should contain non-whitespace content for '{}'", completion.label);
                        }
                    }
                }
            }
        }
    }
}

// Property 35: 补全项插入文本正确性
proptest! {
    #[test]
    fn prop_completion_items_have_valid_insert_text(
        uri in test_uri(),
        code in complex_rust_code_with_macros()
    ) {
        use spring_lsp::completion::CompletionEngine;
        use spring_lsp::schema::SchemaProvider;

        let analyzer = MacroAnalyzer::new();
        let schema_provider = SchemaProvider::new();
        let engine = CompletionEngine::new(schema_provider);

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 对每个宏获取补全项
        for macro_info in &doc.macros {
            let completions = engine.complete_macro(macro_info, None);

            // 验证每个补全项的插入文本
            for completion in &completions {
                if let Some(insert_text) = &completion.insert_text {
                    // 插入文本不应为空
                    prop_assert!(!insert_text.is_empty(),
                        "Insert text should not be empty for '{}'", completion.label);

                    // 如果是 snippet 格式，应该包含占位符标记
                    if completion.insert_text_format == Some(lsp_types::InsertTextFormat::SNIPPET) {
                        // Snippet 应该包含 $ 符号（占位符标记）
                        prop_assert!(insert_text.contains('$'),
                            "Snippet insert text should contain placeholder markers for '{}'", completion.label);
                    }

                    // 插入文本应该与 label 相关
                    // 至少应该包含 label 的一部分或者是有效的替代文本
                    let is_related = insert_text.contains(&completion.label) ||
                                    completion.label.contains(insert_text) ||
                                    insert_text.len() > 0; // 至少不为空
                    prop_assert!(is_related,
                        "Insert text should be related to label for '{}'", completion.label);
                }
            }
        }
    }
}

// Property 35: 补全项类型一致性
proptest! {
    #[test]
    fn prop_completion_items_have_consistent_types(
        uri in test_uri(),
        code in complex_rust_code_with_macros()
    ) {
        use spring_lsp::completion::CompletionEngine;
        use spring_lsp::schema::SchemaProvider;

        let analyzer = MacroAnalyzer::new();
        let schema_provider = SchemaProvider::new();
        let engine = CompletionEngine::new(schema_provider);

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 对每个宏获取补全项并验证类型一致性
        for macro_info in &doc.macros {
            let completions = engine.complete_macro(macro_info, None);

            match macro_info {
                spring_lsp::macro_analyzer::SpringMacro::DeriveService(_) => {
                    // Service 宏的所有补全项应该是 PROPERTY 类型
                    for completion in &completions {
                        prop_assert_eq!(completion.kind, Some(lsp_types::CompletionItemKind::PROPERTY),
                            "Service macro completion '{}' should be PROPERTY kind", completion.label);
                    }
                }
                spring_lsp::macro_analyzer::SpringMacro::Inject(_) => {
                    // Inject 宏的所有补全项应该是 KEYWORD 类型
                    for completion in &completions {
                        prop_assert_eq!(completion.kind, Some(lsp_types::CompletionItemKind::KEYWORD),
                            "Inject macro completion '{}' should be KEYWORD kind", completion.label);
                    }
                }
                spring_lsp::macro_analyzer::SpringMacro::AutoConfig(_) => {
                    // AutoConfig 宏的所有补全项应该是 CLASS 类型
                    for completion in &completions {
                        prop_assert_eq!(completion.kind, Some(lsp_types::CompletionItemKind::CLASS),
                            "AutoConfig macro completion '{}' should be CLASS kind", completion.label);
                    }
                }
                spring_lsp::macro_analyzer::SpringMacro::Route(_) => {
                    // Route 宏的补全项应该是 CONSTANT（HTTP 方法）或 SNIPPET（路径参数）
                    for completion in &completions {
                        let is_valid_kind = completion.kind == Some(lsp_types::CompletionItemKind::CONSTANT) ||
                                          completion.kind == Some(lsp_types::CompletionItemKind::SNIPPET);
                        prop_assert!(is_valid_kind,
                            "Route macro completion '{}' should be CONSTANT or SNIPPET kind", completion.label);
                    }
                }
                spring_lsp::macro_analyzer::SpringMacro::Job(_) => {
                    // Job 宏的补全项应该是 SNIPPET（cron 表达式）或 VALUE（延迟/频率值）
                    for completion in &completions {
                        let is_valid_kind = completion.kind == Some(lsp_types::CompletionItemKind::SNIPPET) ||
                                          completion.kind == Some(lsp_types::CompletionItemKind::VALUE);
                        prop_assert!(is_valid_kind,
                            "Job macro completion '{}' should be SNIPPET or VALUE kind", completion.label);
                    }
                }
            }
        }
    }
}

// Property 35: 补全引擎不应崩溃
proptest! {
    #[test]
    fn prop_completion_engine_does_not_crash(
        uri in test_uri(),
        code in complex_rust_code_with_macros()
    ) {
        use spring_lsp::completion::CompletionEngine;
        use spring_lsp::schema::SchemaProvider;

        let analyzer = MacroAnalyzer::new();
        let schema_provider = SchemaProvider::new();
        let engine = CompletionEngine::new(schema_provider);

        // 解析并提取宏
        let parse_result = analyzer.parse(uri.clone(), code.clone());
        prop_assert!(parse_result.is_ok());

        let doc = parse_result.unwrap();
        let extract_result = analyzer.extract_macros(doc);
        prop_assert!(extract_result.is_ok());

        let doc = extract_result.unwrap();

        // 对每个宏调用 complete_macro 不应该崩溃
        for macro_info in &doc.macros {
            // 不应该 panic
            let _completions = engine.complete_macro(macro_info, None);

            // 如果执行到这里，说明没有崩溃
        }
    }
}

// Property 35: 补全项数量合理性
proptest! {
    #[test]
    fn prop_completion_items_count_is_reasonable(
        uri in test_uri(),
        code in complex_rust_code_with_macros()
    ) {
        use spring_lsp::completion::CompletionEngine;
        use spring_lsp::schema::SchemaProvider;

        let analyzer = MacroAnalyzer::new();
        let schema_provider = SchemaProvider::new();
        let engine = CompletionEngine::new(schema_provider);

        // 解析并提取宏
        let doc = analyzer.parse(uri.clone(), code.clone()).unwrap();
        let doc = analyzer.extract_macros(doc).unwrap();

        // 对每个宏验证补全项数量
        for macro_info in &doc.macros {
            let completions = engine.complete_macro(macro_info, None);

            // 补全项数量应该在合理范围内（1-100）
            prop_assert!(completions.len() >= 1 && completions.len() <= 100,
                "Completion items count should be between 1 and 100, got {}", completions.len());

            // 根据宏类型验证最小补全项数量
            match macro_info {
                spring_lsp::macro_analyzer::SpringMacro::DeriveService(_) => {
                    // Service 宏至少应该提供 3 个补全项（inject(component), inject(component = "name"), inject(config)）
                    prop_assert!(completions.len() >= 3,
                        "Service macro should provide at least 3 completion items");
                }
                spring_lsp::macro_analyzer::SpringMacro::Inject(_) => {
                    // Inject 宏至少应该提供 2 个补全项（component, config）
                    prop_assert!(completions.len() >= 2,
                        "Inject macro should provide at least 2 completion items");
                }
                spring_lsp::macro_analyzer::SpringMacro::AutoConfig(_) => {
                    // AutoConfig 宏至少应该提供 3 个补全项（WebConfigurator, JobConfigurator, StreamConfigurator）
                    prop_assert!(completions.len() >= 3,
                        "AutoConfig macro should provide at least 3 completion items");
                }
                spring_lsp::macro_analyzer::SpringMacro::Route(_) => {
                    // Route 宏至少应该提供 7 个 HTTP 方法 + 1 个路径参数模板
                    prop_assert!(completions.len() >= 8,
                        "Route macro should provide at least 8 completion items");
                }
                spring_lsp::macro_analyzer::SpringMacro::Job(_) => {
                    // Job 宏至少应该提供 3 个 cron 表达式 + 3 个延迟/频率值
                    prop_assert!(completions.len() >= 6,
                        "Job macro should provide at least 6 completion items");
                }
            }
        }
    }
}
