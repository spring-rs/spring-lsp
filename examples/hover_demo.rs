//! 宏悬停提示功能演示
//! 
//! 这个示例展示了如何使用 MacroAnalyzer 的 hover_macro 方法
//! 为 spring-rs 宏生成悬停提示信息

use lsp_types::{Position, Range, Url};
use spring_lsp::macro_analyzer::*;

fn main() {
    println!("=== Spring-rs 宏悬停提示演示 ===\n");
    
    let analyzer = MacroAnalyzer::new();
    
    // 示例 1: Service 宏悬停提示
    println!("1. Service 宏悬停提示");
    println!("-------------------");
    
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
                name: "cache".to_string(),
                type_name: "RedisPool".to_string(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Component,
                    component_name: Some("redis".to_string()),
                    range: test_range(),
                }),
            },
            Field {
                name: "config".to_string(),
                type_name: "UserConfig".to_string(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Config,
                    component_name: None,
                    range: test_range(),
                }),
            },
        ],
        range: test_range(),
    };
    
    let hover = analyzer.hover_macro(&SpringMacro::DeriveService(service));
    println!("{}\n", hover);
    
    // 示例 2: Inject 宏悬停提示（组件注入）
    println!("2. Inject 宏悬停提示 - 组件注入");
    println!("----------------------------");
    
    let inject_component = InjectMacro {
        inject_type: InjectType::Component,
        component_name: Some("primary_db".to_string()),
        range: test_range(),
    };
    
    let hover = analyzer.hover_macro(&SpringMacro::Inject(inject_component));
    println!("{}\n", hover);
    
    // 示例 3: Inject 宏悬停提示（配置注入）
    println!("3. Inject 宏悬停提示 - 配置注入");
    println!("----------------------------");
    
    let inject_config = InjectMacro {
        inject_type: InjectType::Config,
        component_name: None,
        range: test_range(),
    };
    
    let hover = analyzer.hover_macro(&SpringMacro::Inject(inject_config));
    println!("{}\n", hover);
    
    // 示例 4: 路由宏悬停提示
    println!("4. 路由宏悬停提示");
    println!("----------------");
    
    let route = RouteMacro {
        path: "/api/users/{id}".to_string(),
        methods: vec![HttpMethod::Get, HttpMethod::Put],
        middlewares: vec!["AuthMiddleware".to_string(), "LogMiddleware".to_string()],
        handler_name: "handle_user".to_string(),
        range: test_range(),
    };
    
    let hover = analyzer.hover_macro(&SpringMacro::Route(route));
    println!("{}\n", hover);
    
    // 示例 5: AutoConfig 宏悬停提示
    println!("5. AutoConfig 宏悬停提示");
    println!("----------------------");
    
    let auto_config = AutoConfigMacro {
        configurator_type: "WebConfigurator".to_string(),
        range: test_range(),
    };
    
    let hover = analyzer.hover_macro(&SpringMacro::AutoConfig(auto_config));
    println!("{}\n", hover);
    
    // 示例 6: Cron 任务宏悬停提示
    println!("6. Cron 任务宏悬停提示");
    println!("--------------------");
    
    let cron_job = JobMacro::Cron {
        expression: "0 0 * * * *".to_string(),
        range: test_range(),
    };
    
    let hover = analyzer.hover_macro(&SpringMacro::Job(cron_job));
    println!("{}\n", hover);
    
    // 示例 7: FixDelay 任务宏悬停提示
    println!("7. FixDelay 任务宏悬停提示");
    println!("------------------------");
    
    let fix_delay_job = JobMacro::FixDelay {
        seconds: 5,
        range: test_range(),
    };
    
    let hover = analyzer.hover_macro(&SpringMacro::Job(fix_delay_job));
    println!("{}\n", hover);
    
    // 示例 8: FixRate 任务宏悬停提示
    println!("8. FixRate 任务宏悬停提示");
    println!("-----------------------");
    
    let fix_rate_job = JobMacro::FixRate {
        seconds: 10,
        range: test_range(),
    };
    
    let hover = analyzer.hover_macro(&SpringMacro::Job(fix_rate_job));
    println!("{}\n", hover);
    
    // 示例 9: 完整工作流演示
    println!("9. 完整工作流演示");
    println!("----------------");
    println!("解析 Rust 文件 -> 提取宏 -> 生成悬停提示\n");
    
    let uri = Url::parse("file:///example.rs").unwrap();
    let content = r#"
        #[derive(Clone, Service)]
        struct OrderService {
            #[inject(component = "primary")]
            db: ConnectPool,
            
            #[inject(config)]
            config: OrderConfig,
        }
        
        #[get("/orders/{id}")]
        async fn get_order(id: i64) -> Result<Json<Order>> {
            Ok(Json(Order::default()))
        }
    "#.to_string();
    
    // 解析文件
    let doc = analyzer.parse(uri, content).unwrap();
    println!("✓ 文件解析成功");
    
    // 提取宏
    let result = analyzer.extract_macros(doc).unwrap();
    println!("✓ 提取到 {} 个宏", result.macros.len());
    
    // 为每个宏生成悬停提示
    for (i, macro_item) in result.macros.iter().enumerate() {
        println!("\n宏 #{}", i + 1);
        let hover = analyzer.hover_macro(macro_item);
        
        // 显示悬停提示的前几行
        let lines: Vec<&str> = hover.lines().take(5).collect();
        for line in lines {
            println!("  {}", line);
        }
        println!("  ...");
    }
    
    println!("\n=== 演示完成 ===");
}

/// 创建测试用的 Range
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
