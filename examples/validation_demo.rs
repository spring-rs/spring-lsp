//! 宏参数验证功能演示
//! 
//! 这个示例展示了 spring-lsp 如何验证 spring-rs 宏的参数正确性

use lsp_types::Url;
use spring_lsp::macro_analyzer::*;

fn main() {
    println!("=== Spring-LSP 宏参数验证演示 ===\n");
    
    let analyzer = MacroAnalyzer::new();
    
    // 示例 1: 验证有效的 Service 宏
    println!("1. 验证有效的 Service 宏:");
    let valid_service = ServiceMacro {
        struct_name: "UserService".to_string(),
        fields: vec![
            Field {
                name: "db".to_string(),
                type_name: "ConnectPool".to_string(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Component,
                    component_name: None,
                    range: default_range(),
                }),
            },
        ],
        range: default_range(),
    };
    
    let diagnostics = analyzer.validate_macro(&SpringMacro::DeriveService(valid_service));
    if diagnostics.is_empty() {
        println!("   ✓ 验证通过，没有错误\n");
    } else {
        println!("   ✗ 发现 {} 个错误\n", diagnostics.len());
    }
    
    // 示例 2: 验证无效的 Service 宏（空组件名称）
    println!("2. 验证无效的 Service 宏（空组件名称）:");
    let invalid_service = ServiceMacro {
        struct_name: "UserService".to_string(),
        fields: vec![
            Field {
                name: "db".to_string(),
                type_name: "ConnectPool".to_string(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Component,
                    component_name: Some("".to_string()), // 空字符串
                    range: default_range(),
                }),
            },
        ],
        range: default_range(),
    };
    
    let diagnostics = analyzer.validate_macro(&SpringMacro::DeriveService(invalid_service));
    if !diagnostics.is_empty() {
        println!("   ✗ 发现 {} 个错误:", diagnostics.len());
        for diagnostic in &diagnostics {
            println!("      - {}", diagnostic.message);
        }
        println!();
    }
    
    // 示例 3: 验证无效的路由宏（路径不以 / 开头）
    println!("3. 验证无效的路由宏（路径不以 / 开头）:");
    let invalid_route = RouteMacro {
        path: "users".to_string(), // 不以 / 开头
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "get_users".to_string(),
        range: default_range(),
    };
    
    let diagnostics = analyzer.validate_macro(&SpringMacro::Route(invalid_route));
    if !diagnostics.is_empty() {
        println!("   ✗ 发现 {} 个错误:", diagnostics.len());
        for diagnostic in &diagnostics {
            println!("      - {}", diagnostic.message);
        }
        println!();
    }
    
    // 示例 4: 验证无效的路由宏（路径参数格式错误）
    println!("4. 验证无效的路由宏（路径参数格式错误）:");
    let invalid_route_params = RouteMacro {
        path: "/users/{id-name}".to_string(), // 参数名包含非法字符
        methods: vec![HttpMethod::Get],
        middlewares: vec![],
        handler_name: "get_user".to_string(),
        range: default_range(),
    };
    
    let diagnostics = analyzer.validate_macro(&SpringMacro::Route(invalid_route_params));
    if !diagnostics.is_empty() {
        println!("   ✗ 发现 {} 个错误:", diagnostics.len());
        for diagnostic in &diagnostics {
            println!("      - {}", diagnostic.message);
        }
        println!();
    }
    
    // 示例 5: 验证无效的 Cron 任务（表达式格式错误）
    println!("5. 验证无效的 Cron 任务（表达式格式错误）:");
    let invalid_cron = JobMacro::Cron {
        expression: "0 0 *".to_string(), // 只有 3 个部分，应该有 6 个
        range: default_range(),
    };
    
    let diagnostics = analyzer.validate_macro(&SpringMacro::Job(invalid_cron));
    if !diagnostics.is_empty() {
        println!("   ✗ 发现 {} 个错误:", diagnostics.len());
        for diagnostic in &diagnostics {
            println!("      - {}", diagnostic.message);
        }
        println!();
    }
    
    // 示例 6: 验证无效的 FixRate 任务（频率为 0）
    println!("6. 验证无效的 FixRate 任务（频率为 0）:");
    let invalid_fix_rate = JobMacro::FixRate {
        seconds: 0, // 频率不能为 0
        range: default_range(),
    };
    
    let diagnostics = analyzer.validate_macro(&SpringMacro::Job(invalid_fix_rate));
    if !diagnostics.is_empty() {
        println!("   ✗ 发现 {} 个错误:", diagnostics.len());
        for diagnostic in &diagnostics {
            println!("      - {}", diagnostic.message);
        }
        println!();
    }
    
    // 示例 7: 验证完整的 Rust 文件
    println!("7. 验证完整的 Rust 文件:");
    let uri = Url::parse("file:///test.rs").unwrap();
    let content = r#"
        #[derive(Clone, Service)]
        struct UserService {
            #[inject(component = "primary")]
            db: ConnectPool,
            
            #[inject(config)]
            config: UserConfig,
        }
        
        #[get("/users/{id}")]
        async fn get_user(id: i64) -> Result<Json<User>> {
            Ok(Json(User::default()))
        }
        
        #[cron("0 0 * * * *")]
        async fn hourly_cleanup() {
            println!("Cleanup");
        }
    "#.to_string();
    
    match analyzer.parse(uri, content) {
        Ok(doc) => {
            match analyzer.extract_macros(doc) {
                Ok(result) => {
                    println!("   识别到 {} 个宏", result.macros.len());
                    
                    let mut total_errors = 0;
                    for macro_item in &result.macros {
                        let diagnostics = analyzer.validate_macro(macro_item);
                        total_errors += diagnostics.len();
                    }
                    
                    if total_errors == 0 {
                        println!("   ✓ 所有宏验证通过，没有错误\n");
                    } else {
                        println!("   ✗ 发现 {} 个错误\n", total_errors);
                    }
                }
                Err(e) => {
                    println!("   ✗ 提取宏失败: {}\n", e);
                }
            }
        }
        Err(e) => {
            println!("   ✗ 解析失败: {}\n", e);
        }
    }
    
    println!("=== 演示完成 ===");
}

/// 创建默认的 Range
fn default_range() -> lsp_types::Range {
    lsp_types::Range {
        start: lsp_types::Position {
            line: 0,
            character: 0,
        },
        end: lsp_types::Position {
            line: 0,
            character: 10,
        },
    }
}
