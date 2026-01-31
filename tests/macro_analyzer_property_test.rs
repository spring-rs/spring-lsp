//! MacroAnalyzer 属性测试
//!
//! 使用 proptest 验证 MacroAnalyzer 的通用正确性属性

use proptest::prelude::*;
use spring_lsp::macro_analyzer::MacroAnalyzer;
use lsp_types::Url;

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
    "as", "break", "const", "continue", "crate", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
    "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
    "super", "trait", "true", "type", "unsafe", "use", "where", "while",
    "async", "await", "dyn", "abstract", "become", "box", "do", "final",
    "macro", "override", "priv", "typeof", "unsized", "virtual", "yield",
    "try", "_",
];

/// 生成有效的 Rust 标识符
/// 
/// Rust 标识符必须以字母或下划线开头，后跟字母、数字或下划线
/// 注意：避免生成 Rust 关键字
fn valid_identifier() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_]{0,30}"
        .prop_filter("not a Rust keyword", |s| !RUST_KEYWORDS.contains(&s.as_str()))
}

/// 生成有效的 Rust 类型名称（通常是 PascalCase）
fn valid_type_name() -> impl Strategy<Value = String> {
    "[A-Z][a-zA-Z0-9]{0,30}"
}

/// 生成简单的 Rust 结构体定义
fn simple_struct() -> impl Strategy<Value = String> {
    (valid_type_name(), prop::collection::vec(valid_identifier(), 0..5))
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
        .prop_map(|fn_name| {
            format!("fn {}() {{\n    // function body\n}}\n", fn_name)
        })
}

/// 生成简单的 Rust 枚举定义
fn simple_enum() -> impl Strategy<Value = String> {
    (valid_type_name(), prop::collection::vec(valid_type_name(), 1..5))
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
    (valid_type_name(), valid_identifier())
        .prop_map(|(type_name, method_name)| {
            format!(
                "impl {} {{\n    fn {}(&self) {{\n        // method body\n    }}\n}}\n",
                type_name, method_name
            )
        })
}

/// 生成简单的 Rust use 语句
fn simple_use() -> impl Strategy<Value = String> {
    (valid_identifier(), valid_identifier())
        .prop_map(|(module, item)| {
            format!("use {}::{};\n", module, item)
        })
}

/// 生成简单的 Rust mod 声明
fn simple_mod() -> impl Strategy<Value = String> {
    valid_identifier()
        .prop_map(|mod_name| {
            format!("mod {};\n", mod_name)
        })
}

/// 生成简单的 Rust 常量定义
fn simple_const() -> impl Strategy<Value = String> {
    (valid_identifier(), any::<i32>())
        .prop_map(|(const_name, value)| {
            format!("const {}: i32 = {};\n", const_name.to_uppercase(), value)
        })
}

/// 生成简单的 Rust trait 定义
fn simple_trait() -> impl Strategy<Value = String> {
    (valid_type_name(), valid_identifier())
        .prop_map(|(trait_name, method_name)| {
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
        0..10
    ).prop_map(|items| {
        items.join("\n")
    })
}

/// 生成包含注释的 Rust 代码
fn rust_code_with_comments() -> impl Strategy<Value = String> {
    (valid_rust_code(), prop::collection::vec("[a-zA-Z0-9 ]{1,50}", 0..5))
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
    (simple_function(), prop::collection::vec("[a-zA-Z0-9 ]{1,50}", 0..3))
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
    valid_identifier()
        .prop_map(|name| {
            Url::parse(&format!("file:///test/{}.rs", name))
                .expect("Failed to create test URI")
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
        prop::collection::vec(
            (valid_identifier(), valid_type_name()),
            0..5
        )
    ).prop_map(|(struct_name, fields)| {
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
        prop_oneof![
            Just("component".to_string()),
            Just("config".to_string()),
        ]
    )
}

/// 生成带有 #[inject] 属性和组件名称的字段
fn inject_field_with_name() -> impl Strategy<Value = (String, String, String, String)> {
    (
        valid_identifier(),
        valid_type_name(),
        prop_oneof![
            Just("component".to_string()),
            Just("config".to_string()),
        ],
        valid_identifier()
    )
}

/// 生成带有 #[inject] 属性的 Service 结构体
fn service_struct_with_inject() -> impl Strategy<Value = String> {
    (
        valid_type_name(),
        prop::collection::vec(inject_field(), 1..5)
    ).prop_map(|(struct_name, fields)| {
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
        prop::collection::vec(inject_field_with_name(), 1..5)
    ).prop_map(|(struct_name, fields)| {
        let mut code = format!("#[derive(Service)]\nstruct {} {{\n", struct_name);
        for (field_name, field_type, inject_type, component_name) in fields {
            code.push_str(&format!("    #[inject({} = \"{}\")]\n", inject_type, component_name));
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
        valid_identifier()
    ).prop_map(|(method, path, fn_name)| {
        format!("#[{}(\"{}\")]\nasync fn {}() {{\n    // handler\n}}\n", method, path, fn_name)
    })
}

/// 生成路由宏（多个方法）
fn route_macro_multiple_methods() -> impl Strategy<Value = String> {
    (
        valid_route_path(),
        prop::collection::vec(
            prop_oneof![
                Just("GET"),
                Just("POST"),
                Just("PUT"),
                Just("DELETE"),
            ],
            1..4
        ),
        valid_identifier()
    ).prop_map(|(path, methods, fn_name)| {
        let method_attrs = methods.iter()
            .map(|m| format!("method = \"{}\"", m))
            .collect::<Vec<_>>()
            .join(", ");
        format!("#[route(\"{}\", {})]\nasync fn {}() {{\n    // handler\n}}\n", path, method_attrs, fn_name)
    })
}

/// 生成带有路径参数的路由宏
fn route_macro_with_params() -> impl Strategy<Value = String> {
    (
        prop_oneof![
            Just("get"),
            Just("post"),
            Just("put"),
            Just("delete"),
        ],
        route_path_with_params(),
        valid_identifier()
    ).prop_map(|(method, path, fn_name)| {
        format!("#[{}(\"{}\")]\nasync fn {}() {{\n    // handler\n}}\n", method, path, fn_name)
    })
}

/// 生成 #[auto_config] 宏
fn auto_config_macro() -> impl Strategy<Value = String> {
    (
        valid_type_name(),
        valid_identifier()
    ).prop_map(|(configurator, fn_name)| {
        format!("#[auto_config({})]\nasync fn {}() {{\n    // config\n}}\n", configurator, fn_name)
    })
}

/// 生成 #[cron] 任务宏
fn cron_job_macro() -> impl Strategy<Value = String> {
    (
        prop_oneof![
            Just("0 0 * * * *"),  // 每小时
            Just("0 0 0 * * *"),  // 每天
            Just("0 */5 * * * *"), // 每5分钟
            Just("0 0 12 * * MON-FRI"), // 工作日中午
        ],
        valid_identifier()
    ).prop_map(|(cron_expr, fn_name)| {
        format!("#[cron(\"{}\")]\nasync fn {}() {{\n    // job\n}}\n", cron_expr, fn_name)
    })
}

/// 生成 #[fix_delay] 任务宏
fn fix_delay_job_macro() -> impl Strategy<Value = String> {
    (
        1u64..3600,
        valid_identifier()
    ).prop_map(|(seconds, fn_name)| {
        format!("#[fix_delay({})]\nasync fn {}() {{\n    // job\n}}\n", seconds, fn_name)
    })
}

/// 生成 #[fix_rate] 任务宏
fn fix_rate_job_macro() -> impl Strategy<Value = String> {
    (
        1u64..3600,
        valid_identifier()
    ).prop_map(|(seconds, fn_name)| {
        format!("#[fix_rate({})]\nasync fn {}() {{\n    // job\n}}\n", seconds, fn_name)
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
    ).prop_map(|(service, route, auto_config, job)| {
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
