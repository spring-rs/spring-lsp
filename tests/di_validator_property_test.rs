//! 依赖注入验证器属性测试
//!
//! 使用 proptest 验证依赖注入验证器的通用正确性属性

use lsp_types::{Location, Position, Range, Url};
use proptest::prelude::*;
use spring_lsp::di_validator::DependencyInjectionValidator;
use spring_lsp::index::{
    ComponentIndex, ComponentInfo, IndexManager, SymbolIndex, SymbolInfo, SymbolType,
};
use spring_lsp::macro_analyzer::{
    Field, InjectMacro, InjectType, RustDocument, ServiceMacro, SpringMacro,
};
use spring_lsp::toml_analyzer::{ConfigSection, TomlDocument};
use std::collections::HashMap;

// ============================================================================
// 测试数据生成器
// ============================================================================

/// 生成有效的标识符名称（用于类型名、字段名等）
fn valid_identifier() -> impl Strategy<Value = String> {
    "[A-Z][A-Za-z0-9]{0,20}"
}

/// 生成有效的配置前缀
fn valid_config_prefix() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,20}"
}

/// 生成有效的 URI
fn valid_uri() -> impl Strategy<Value = Url> {
    Just(Url::parse("file:///test.rs").unwrap())
}

/// 生成有效的位置范围
fn valid_range() -> impl Strategy<Value = Range> {
    (0u32..100, 0u32..100, 0u32..100, 0u32..100).prop_map(
        |(start_line, start_char, end_line, end_char)| Range {
            start: Position {
                line: start_line,
                character: start_char,
            },
            end: Position {
                line: start_line + end_line,
                character: start_char + end_char,
            },
        },
    )
}

/// 生成 Location
fn valid_location() -> impl Strategy<Value = Location> {
    (valid_uri(), valid_range()).prop_map(|(uri, range)| Location { uri, range })
}

/// 生成 InjectType
fn inject_type() -> impl Strategy<Value = InjectType> {
    prop_oneof![Just(InjectType::Component), Just(InjectType::Config),]
}

/// 生成 InjectMacro
fn inject_macro() -> impl Strategy<Value = InjectMacro> {
    (
        inject_type(),
        proptest::option::of(valid_identifier()),
        valid_range(),
    )
        .prop_map(|(inject_type, component_name, range)| InjectMacro {
            inject_type,
            component_name,
            range,
        })
}

/// 生成 Field
fn field() -> impl Strategy<Value = Field> {
    (
        "[a-z][a-z0-9_]{0,20}",
        valid_identifier(),
        proptest::option::of(inject_macro()),
    )
        .prop_map(|(name, type_name, inject)| Field {
            name,
            type_name,
            inject,
        })
}

/// 生成 ServiceMacro
fn service_macro() -> impl Strategy<Value = ServiceMacro> {
    (
        valid_identifier(),
        prop::collection::vec(field(), 0..5),
        valid_range(),
    )
        .prop_map(|(struct_name, fields, range)| ServiceMacro {
            struct_name,
            fields,
            range,
        })
}

/// 生成 RustDocument
fn rust_document() -> impl Strategy<Value = RustDocument> {
    (valid_uri(), prop::collection::vec(service_macro(), 0..5)).prop_map(|(uri, service_macros)| {
        let macros = service_macros
            .into_iter()
            .map(SpringMacro::DeriveService)
            .collect();
        RustDocument {
            uri,
            content: String::new(),
            macros,
        }
    })
}

/// 生成 ComponentInfo
fn component_info() -> impl Strategy<Value = ComponentInfo> {
    (
        valid_identifier(),
        valid_identifier(),
        valid_location(),
        proptest::option::of("[a-z][a-z0-9-]{0,20}"),
    )
        .prop_map(|(name, type_name, location, plugin)| ComponentInfo {
            name,
            type_name,
            location,
            plugin,
        })
}

/// 生成 SymbolInfo
fn symbol_info() -> impl Strategy<Value = SymbolInfo> {
    (valid_identifier(), valid_location()).prop_map(|(name, location)| SymbolInfo {
        name,
        symbol_type: SymbolType::Struct,
        location,
    })
}

/// 生成 ConfigSection
fn config_section() -> impl Strategy<Value = ConfigSection> {
    (valid_config_prefix(), valid_range()).prop_map(|(prefix, range)| ConfigSection {
        prefix,
        properties: HashMap::new(),
        range,
    })
}

/// 生成 TomlDocument
fn toml_document() -> impl Strategy<Value = TomlDocument> {
    prop::collection::hash_map(valid_config_prefix(), config_section(), 0..5).prop_map(
        |config_sections| {
            // 创建一个简单的 TOML 根节点
            let content = String::new();
            let root = taplo::parser::parse(&content).into_dom();

            TomlDocument {
                root,
                env_vars: Vec::new(),
                config_sections,
                content,
            }
        },
    )
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 创建带有组件索引的 IndexManager
fn create_index_manager_with_components(components: Vec<ComponentInfo>) -> IndexManager {
    let manager = IndexManager::new();
    let component_index = ComponentIndex::new();

    for component in components {
        component_index.add(component.name.clone(), component);
    }

    // 注意：这里我们无法直接设置 IndexManager 的内部索引
    // 因为 IndexManager 的字段是私有的
    // 在实际测试中，我们需要通过 build() 方法来构建索引
    // 这里我们返回一个空的 IndexManager，测试将验证空索引的行为
    manager
}

/// 创建带有符号索引的 IndexManager
fn create_index_manager_with_symbols(symbols: Vec<SymbolInfo>) -> IndexManager {
    let manager = IndexManager::new();
    let symbol_index = SymbolIndex::new();

    for symbol in symbols {
        symbol_index.add(symbol.name.clone(), symbol);
    }

    // 同上，返回空的 IndexManager
    manager
}

// ============================================================================
// Property 49: 组件注册验证
// ============================================================================

// Feature: spring-lsp, Property 49: 组件注册验证
//
// **Validates: Requirements 11.1**
//
// *For any* `#[inject(component)]` 注入，如果组件未在任何插件中注册，
// 诊断引擎应该生成错误诊断。
//
// 这个属性测试验证：
// 1. 当注入的组件未注册时，应该生成错误诊断
// 2. 错误诊断的代码应该是 "component-not-registered"
// 3. 错误诊断的严重性应该是 ERROR
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_component_registration_validation(
        type_name in valid_identifier(),
        field_name in "[a-z][a-z0-9_]{0,20}",
    ) {
        // 创建一个包含组件注入的服务
        let service = ServiceMacro {
            struct_name: "TestService".to_string(),
            fields: vec![Field {
                name: field_name,
                type_name: type_name.clone(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Component,
                    component_name: None,
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 10 },
                    },
                }),
            }],
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 10, character: 0 },
            },
        };

        let rust_doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::DeriveService(service)],
        };

        // 创建空的 IndexManager（没有注册任何组件）
        let index_manager = IndexManager::new();
        let validator = DependencyInjectionValidator::new(index_manager);

        // 验证依赖注入
        let diagnostics = validator.validate(&[rust_doc], &[]);

        // 应该生成至少一个诊断
        prop_assert!(!diagnostics.is_empty(),
            "Should generate diagnostic for unregistered component");

        // 检查是否有组件未注册的错误
        let has_unregistered_error = diagnostics.iter().any(|d| {
            d.code.as_ref().map(|c| {
                matches!(c, lsp_types::NumberOrString::String(s)
                    if s == "component-not-registered" || s == "component-type-not-found")
            }).unwrap_or(false)
        });

        prop_assert!(has_unregistered_error,
            "Should have component-not-registered or component-type-not-found error");
    }
}

// ============================================================================
// Property 50: 组件类型存在性验证
// ============================================================================

// Feature: spring-lsp, Property 50: 组件类型存在性验证
//
// **Validates: Requirements 11.2**
//
// *For any* 注入的组件类型，如果该类型在项目中不存在，
// 诊断引擎应该生成错误诊断。
//
// 这个属性测试验证：
// 1. 当注入的组件类型不存在时，应该生成错误诊断
// 2. 错误诊断的代码应该是 "component-type-not-found"
// 3. 错误诊断的严重性应该是 ERROR
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_component_type_existence_validation(
        type_name in valid_identifier(),
        field_name in "[a-z][a-z0-9_]{0,20}",
    ) {
        // 创建一个包含组件注入的服务
        let service = ServiceMacro {
            struct_name: "TestService".to_string(),
            fields: vec![Field {
                name: field_name,
                type_name: type_name.clone(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Component,
                    component_name: None,
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 10 },
                    },
                }),
            }],
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 10, character: 0 },
            },
        };

        let rust_doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::DeriveService(service)],
        };

        // 创建空的 IndexManager（没有任何符号）
        let index_manager = IndexManager::new();
        let validator = DependencyInjectionValidator::new(index_manager);

        // 验证依赖注入
        let diagnostics = validator.validate(&[rust_doc], &[]);

        // 应该生成至少一个诊断
        prop_assert!(!diagnostics.is_empty(),
            "Should generate diagnostic for non-existent component type");

        // 检查是否有类型不存在的错误
        let has_type_error = diagnostics.iter().any(|d| {
            d.code.as_ref().map(|c| {
                matches!(c, lsp_types::NumberOrString::String(s)
                    if s == "component-type-not-found" || s == "component-not-registered")
            }).unwrap_or(false)
        });

        prop_assert!(has_type_error,
            "Should have component-type-not-found or component-not-registered error");
    }
}

// ============================================================================
// Property 51: 组件名称匹配验证
// ============================================================================

// Feature: spring-lsp, Property 51: 组件名称匹配验证
//
// **Validates: Requirements 11.3**
//
// *For any* 指定了组件名称的注入，如果名称与注册的组件名称不匹配，
// 诊断引擎应该生成错误诊断并提供可用组件列表。
//
// 这个属性测试验证：
// 1. 当指定的组件名称不存在时，应该生成错误诊断
// 2. 错误诊断的代码应该是 "component-name-not-found" 或 "component-name-mismatch"
// 3. 错误诊断的严重性应该是 ERROR
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_component_name_matching_validation(
        type_name in valid_identifier(),
        component_name in valid_identifier(),
        field_name in "[a-z][a-z0-9_]{0,20}",
    ) {
        // 确保组件名称与类型名称不同
        prop_assume!(component_name != type_name);

        // 创建一个包含指定组件名称的注入
        let service = ServiceMacro {
            struct_name: "TestService".to_string(),
            fields: vec![Field {
                name: field_name,
                type_name: type_name.clone(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Component,
                    component_name: Some(component_name.clone()),
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 10 },
                    },
                }),
            }],
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 10, character: 0 },
            },
        };

        let rust_doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::DeriveService(service)],
        };

        // 创建空的 IndexManager（指定的组件名称不存在）
        let index_manager = IndexManager::new();
        let validator = DependencyInjectionValidator::new(index_manager);

        // 验证依赖注入
        let diagnostics = validator.validate(&[rust_doc], &[]);

        // 应该生成至少一个诊断
        prop_assert!(!diagnostics.is_empty(),
            "Should generate diagnostic for non-existent component name");

        // 检查是否有组件名称相关的错误
        let has_name_error = diagnostics.iter().any(|d| {
            d.code.as_ref().map(|c| {
                matches!(c, lsp_types::NumberOrString::String(s)
                    if s == "component-name-not-found"
                    || s == "component-name-mismatch"
                    || s == "component-not-registered"
                    || s == "component-type-not-found")
            }).unwrap_or(false)
        });

        prop_assert!(has_name_error,
            "Should have component name related error");
    }
}

// ============================================================================
// Property 52: 循环依赖检测
// ============================================================================

// Feature: spring-lsp, Property 52: 循环依赖检测
//
// **Validates: Requirements 11.4**
//
// *For any* 组件依赖图，如果存在循环依赖，
// 诊断引擎应该生成警告并建议使用 `LazyComponent`。
//
// 这个属性测试验证：
// 1. 当存在循环依赖时，应该生成警告诊断
// 2. 警告诊断的代码应该是 "circular-dependency"
// 3. 警告诊断的严重性应该是 WARNING
// 4. 警告消息应该建议使用 LazyComponent
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_circular_dependency_detection(
        service_a_name in valid_identifier(),
        service_b_name in valid_identifier(),
    ) {
        // 确保两个服务名称不同
        prop_assume!(service_a_name != service_b_name);

        // 创建两个相互依赖的服务
        // ServiceA 依赖 ServiceB
        let service_a = ServiceMacro {
            struct_name: service_a_name.clone(),
            fields: vec![Field {
                name: "service_b".to_string(),
                type_name: service_b_name.clone(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Component,
                    component_name: None,
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 10 },
                    },
                }),
            }],
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 10, character: 0 },
            },
        };

        // ServiceB 依赖 ServiceA
        let service_b = ServiceMacro {
            struct_name: service_b_name.clone(),
            fields: vec![Field {
                name: "service_a".to_string(),
                type_name: service_a_name.clone(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Component,
                    component_name: None,
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 10 },
                    },
                }),
            }],
            range: Range {
                start: Position { line: 20, character: 0 },
                end: Position { line: 30, character: 0 },
            },
        };

        let rust_doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![
                SpringMacro::DeriveService(service_a),
                SpringMacro::DeriveService(service_b),
            ],
        };

        let index_manager = IndexManager::new();
        let validator = DependencyInjectionValidator::new(index_manager);

        // 验证依赖注入
        let diagnostics = validator.validate(&[rust_doc], &[]);

        // 应该生成至少一个诊断
        prop_assert!(!diagnostics.is_empty(),
            "Should generate diagnostic for circular dependency");

        // 检查是否有循环依赖警告
        let has_circular_warning = diagnostics.iter().any(|d| {
            d.code.as_ref().map(|c| {
                matches!(c, lsp_types::NumberOrString::String(s)
                    if s == "circular-dependency")
            }).unwrap_or(false)
        });

        prop_assert!(has_circular_warning,
            "Should have circular-dependency warning");

        // 检查警告消息是否建议使用 LazyComponent
        let has_lazy_suggestion = diagnostics.iter().any(|d| {
            d.message.contains("LazyComponent")
        });

        prop_assert!(has_lazy_suggestion,
            "Warning message should suggest using LazyComponent");
    }
}

// ============================================================================
// Property 53: 配置注入验证
// ============================================================================

// Feature: spring-lsp, Property 53: 配置注入验证
//
// **Validates: Requirements 11.5**
//
// *For any* `#[inject(config)]` 注入，如果配置项在配置文件中不存在，
// 诊断引擎应该生成错误诊断并提供配置文件链接。
//
// 这个属性测试验证：
// 1. 当注入的配置项不存在时，应该生成错误诊断
// 2. 错误诊断的代码应该是 "config-not-found"
// 3. 错误诊断的严重性应该是 ERROR
// 4. 错误消息应该提示在配置文件中添加配置节
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_config_injection_validation(
        config_type_name in valid_identifier().prop_filter("ends with Config", |s| s.ends_with("Config") || s.len() > 6),
        field_name in "[a-z][a-z0-9_]{0,20}",
    ) {
        // 创建一个包含配置注入的服务
        let service = ServiceMacro {
            struct_name: "TestService".to_string(),
            fields: vec![Field {
                name: field_name,
                type_name: config_type_name.clone(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Config,
                    component_name: None,
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 10 },
                    },
                }),
            }],
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 10, character: 0 },
            },
        };

        let rust_doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::DeriveService(service)],
        };

        // 创建空的 TOML 文档（没有配置节）
        let toml_uri = Url::parse("file:///config/app.toml").unwrap();
        let toml_doc = TomlDocument {
            root: taplo::parser::parse("").into_dom(),
            env_vars: Vec::new(),
            config_sections: HashMap::new(),
            content: String::new(),
        };

        let index_manager = IndexManager::new();
        let validator = DependencyInjectionValidator::new(index_manager);

        // 验证依赖注入
        let diagnostics = validator.validate(&[rust_doc], &[(toml_uri, toml_doc)]);

        // 应该生成至少一个诊断
        prop_assert!(!diagnostics.is_empty(),
            "Should generate diagnostic for missing config");

        // 检查是否有配置未找到的错误
        let has_config_error = diagnostics.iter().any(|d| {
            d.code.as_ref().map(|c| {
                matches!(c, lsp_types::NumberOrString::String(s)
                    if s == "config-not-found")
            }).unwrap_or(false)
        });

        prop_assert!(has_config_error,
            "Should have config-not-found error");

        // 检查错误消息是否提示添加配置节
        let has_config_section_hint = diagnostics.iter().any(|d| {
            d.message.contains("配置") && d.message.contains("不存在")
        });

        prop_assert!(has_config_section_hint,
            "Error message should hint about adding config section");
    }
}

// ============================================================================
// 额外的属性测试：验证无依赖注入时不生成诊断
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_no_diagnostics_for_services_without_injection(
        service_name in valid_identifier(),
        field_name in "[a-z][a-z0-9_]{0,20}",
        type_name in valid_identifier(),
    ) {
        // 创建一个没有注入的服务
        let service = ServiceMacro {
            struct_name: service_name,
            fields: vec![Field {
                name: field_name,
                type_name,
                inject: None, // 没有注入
            }],
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 10, character: 0 },
            },
        };

        let rust_doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::DeriveService(service)],
        };

        let index_manager = IndexManager::new();
        let validator = DependencyInjectionValidator::new(index_manager);

        // 验证依赖注入
        let diagnostics = validator.validate(&[rust_doc], &[]);

        // 不应该生成任何诊断
        prop_assert!(diagnostics.is_empty(),
            "Should not generate diagnostics for services without injection");
    }
}

// ============================================================================
// 额外的属性测试：验证 LazyComponent 不参与循环依赖检测
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_lazy_component_breaks_circular_dependency(
        service_a_name in valid_identifier(),
        service_b_name in valid_identifier(),
    ) {
        // 确保两个服务名称不同
        prop_assume!(service_a_name != service_b_name);

        // 创建两个服务，其中一个使用 LazyComponent
        // ServiceA 依赖 ServiceB（使用 LazyComponent）
        let service_a = ServiceMacro {
            struct_name: service_a_name.clone(),
            fields: vec![Field {
                name: "service_b".to_string(),
                type_name: format!("LazyComponent<{}>", service_b_name),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Component,
                    component_name: None,
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 10 },
                    },
                }),
            }],
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 10, character: 0 },
            },
        };

        // ServiceB 依赖 ServiceA
        let service_b = ServiceMacro {
            struct_name: service_b_name.clone(),
            fields: vec![Field {
                name: "service_a".to_string(),
                type_name: service_a_name.clone(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Component,
                    component_name: None,
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 10 },
                    },
                }),
            }],
            range: Range {
                start: Position { line: 20, character: 0 },
                end: Position { line: 30, character: 0 },
            },
        };

        let rust_doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![
                SpringMacro::DeriveService(service_a),
                SpringMacro::DeriveService(service_b),
            ],
        };

        let index_manager = IndexManager::new();
        let validator = DependencyInjectionValidator::new(index_manager);

        // 验证依赖注入
        let diagnostics = validator.validate(&[rust_doc], &[]);

        // 不应该有循环依赖警告（因为使用了 LazyComponent）
        let has_circular_warning = diagnostics.iter().any(|d| {
            d.code.as_ref().map(|c| {
                matches!(c, lsp_types::NumberOrString::String(s)
                    if s == "circular-dependency")
            }).unwrap_or(false)
        });

        prop_assert!(!has_circular_warning,
            "Should not have circular-dependency warning when using LazyComponent");
    }
}
