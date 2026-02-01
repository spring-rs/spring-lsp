//! 宏展开功能演示
//!
//! 这个示例展示了 MacroAnalyzer 如何展开 spring-rs 宏

use lsp_types::{Position, Range, Url};
use spring_lsp::macro_analyzer::{
    AutoConfigMacro, Field, HttpMethod, InjectMacro, InjectType, JobMacro, MacroAnalyzer,
    RouteMacro, ServiceMacro, SpringMacro,
};

fn main() {
    let analyzer = MacroAnalyzer::new();

    println!("=== Spring-rs 宏展开演示 ===\n");

    // 1. Service 宏展开
    println!("1. Service 宏展开");
    println!("================\n");

    let service = ServiceMacro {
        struct_name: "UserService".to_string(),
        fields: vec![
            Field {
                name: "db".to_string(),
                type_name: "ConnectPool".to_string(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Component,
                    component_name: Some("primary".to_string()),
                    range: default_range(),
                }),
            },
            Field {
                name: "config".to_string(),
                type_name: "UserConfig".to_string(),
                inject: Some(InjectMacro {
                    inject_type: InjectType::Config,
                    component_name: None,
                    range: default_range(),
                }),
            },
        ],
        range: default_range(),
    };

    let expanded = analyzer.expand_macro(&SpringMacro::DeriveService(service));
    println!("{}\n", expanded);

    // 2. 路由宏展开
    println!("2. 路由宏展开");
    println!("=============\n");

    let route = RouteMacro {
        path: "/users/{id}".to_string(),
        methods: vec![HttpMethod::Get],
        middlewares: vec!["AuthMiddleware".to_string(), "LogMiddleware".to_string()],
        handler_name: "get_user".to_string(),
        range: default_range(),
    };

    let expanded = analyzer.expand_macro(&SpringMacro::Route(route));
    println!("{}\n", expanded);

    // 3. AutoConfig 宏展开
    println!("3. AutoConfig 宏展开");
    println!("===================\n");

    let auto_config = AutoConfigMacro {
        configurator_type: "WebConfigurator".to_string(),
        range: default_range(),
    };

    let expanded = analyzer.expand_macro(&SpringMacro::AutoConfig(auto_config));
    println!("{}\n", expanded);

    // 4. Cron 任务宏展开
    println!("4. Cron 任务宏展开");
    println!("==================\n");

    let job = JobMacro::Cron {
        expression: "0 0 * * * *".to_string(),
        range: default_range(),
    };

    let expanded = analyzer.expand_macro(&SpringMacro::Job(job));
    println!("{}\n", expanded);

    // 5. FixDelay 任务宏展开
    println!("5. FixDelay 任务宏展开");
    println!("======================\n");

    let job = JobMacro::FixDelay {
        seconds: 5,
        range: default_range(),
    };

    let expanded = analyzer.expand_macro(&SpringMacro::Job(job));
    println!("{}\n", expanded);

    // 6. 完整示例：解析并展开真实代码
    println!("6. 完整示例：解析并展开真实代码");
    println!("================================\n");

    let code = r#"
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
    "#;

    let uri = Url::parse("file:///example.rs").unwrap();
    let doc = analyzer.parse(uri, code.to_string()).unwrap();
    let result = analyzer.extract_macros(doc).unwrap();

    println!("识别到 {} 个宏:\n", result.macros.len());

    for (i, macro_item) in result.macros.iter().enumerate() {
        println!("宏 #{}", i + 1);
        println!("------");
        let expanded = analyzer.expand_macro(macro_item);
        println!("{}\n", expanded);
    }
}

fn default_range() -> Range {
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
