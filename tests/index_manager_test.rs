//! 索引管理器单元测试

use lsp_types::{Location, Position, Range, Url};
use spring_lsp::index::{
    ComponentIndex, ComponentInfo, IndexManager, SymbolIndex, SymbolInfo, SymbolType, Workspace,
};

#[test]
fn test_symbol_index_new() {
    let index = SymbolIndex::new();
    assert_eq!(index.symbols.len(), 0);
}

#[test]
fn test_symbol_index_add_and_find() {
    let index = SymbolIndex::new();

    let symbol = SymbolInfo {
        name: "User".to_string(),
        symbol_type: SymbolType::Struct,
        location: Location {
            uri: Url::parse("file:///test.rs").unwrap(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 20,
                    character: 0,
                },
            },
        },
    };

    index.add("User".to_string(), symbol.clone());

    let found = index.find("User");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "User");
    assert_eq!(found[0].symbol_type, SymbolType::Struct);
}

#[test]
fn test_symbol_index_find_nonexistent() {
    let index = SymbolIndex::new();
    let found = index.find("NonExistent");
    assert_eq!(found.len(), 0);
}

#[test]
fn test_symbol_index_multiple_symbols_same_name() {
    let index = SymbolIndex::new();

    let symbol1 = SymbolInfo {
        name: "User".to_string(),
        symbol_type: SymbolType::Struct,
        location: Location {
            uri: Url::parse("file:///models.rs").unwrap(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 20,
                    character: 0,
                },
            },
        },
    };

    let symbol2 = SymbolInfo {
        name: "User".to_string(),
        symbol_type: SymbolType::Function,
        location: Location {
            uri: Url::parse("file:///handlers.rs").unwrap(),
            range: Range {
                start: Position {
                    line: 30,
                    character: 0,
                },
                end: Position {
                    line: 40,
                    character: 0,
                },
            },
        },
    };

    index.add("User".to_string(), symbol1);
    index.add("User".to_string(), symbol2);

    let found = index.find("User");
    assert_eq!(found.len(), 2);
}

#[test]
fn test_symbol_index_clear() {
    let index = SymbolIndex::new();

    let symbol = SymbolInfo {
        name: "User".to_string(),
        symbol_type: SymbolType::Struct,
        location: Location {
            uri: Url::parse("file:///test.rs").unwrap(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 20,
                    character: 0,
                },
            },
        },
    };

    index.add("User".to_string(), symbol);
    assert_eq!(index.symbols.len(), 1);

    index.clear();
    assert_eq!(index.symbols.len(), 0);
}

#[test]
fn test_component_index_new() {
    let index = ComponentIndex::new();
    assert_eq!(index.components.len(), 0);
}

#[test]
fn test_component_index_add_and_find() {
    let index = ComponentIndex::new();

    let component = ComponentInfo {
        name: "db".to_string(),
        type_name: "ConnectPool".to_string(),
        location: Location {
            uri: Url::parse("file:///main.rs").unwrap(),
            range: Range {
                start: Position {
                    line: 15,
                    character: 0,
                },
                end: Position {
                    line: 20,
                    character: 0,
                },
            },
        },
        plugin: Some("spring-sqlx".to_string()),
    };

    index.add("db".to_string(), component.clone());

    let found = index.find("db");
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.name, "db");
    assert_eq!(found.type_name, "ConnectPool");
    assert_eq!(found.plugin, Some("spring-sqlx".to_string()));
}

#[test]
fn test_component_index_find_nonexistent() {
    let index = ComponentIndex::new();
    let found = index.find("nonexistent");
    assert!(found.is_none());
}

#[test]
fn test_component_index_replace() {
    let index = ComponentIndex::new();

    let component1 = ComponentInfo {
        name: "db".to_string(),
        type_name: "ConnectPool".to_string(),
        location: Location {
            uri: Url::parse("file:///main.rs").unwrap(),
            range: Range {
                start: Position {
                    line: 15,
                    character: 0,
                },
                end: Position {
                    line: 20,
                    character: 0,
                },
            },
        },
        plugin: Some("spring-sqlx".to_string()),
    };

    let component2 = ComponentInfo {
        name: "db".to_string(),
        type_name: "PostgresPool".to_string(),
        location: Location {
            uri: Url::parse("file:///main.rs").unwrap(),
            range: Range {
                start: Position {
                    line: 25,
                    character: 0,
                },
                end: Position {
                    line: 30,
                    character: 0,
                },
            },
        },
        plugin: Some("spring-postgres".to_string()),
    };

    index.add("db".to_string(), component1);
    index.add("db".to_string(), component2);

    let found = index.find("db");
    assert!(found.is_some());
    let found = found.unwrap();
    // 应该是最后添加的组件
    assert_eq!(found.type_name, "PostgresPool");
    assert_eq!(found.plugin, Some("spring-postgres".to_string()));
}

#[test]
fn test_component_index_clear() {
    let index = ComponentIndex::new();

    let component = ComponentInfo {
        name: "db".to_string(),
        type_name: "ConnectPool".to_string(),
        location: Location {
            uri: Url::parse("file:///main.rs").unwrap(),
            range: Range {
                start: Position {
                    line: 15,
                    character: 0,
                },
                end: Position {
                    line: 20,
                    character: 0,
                },
            },
        },
        plugin: Some("spring-sqlx".to_string()),
    };

    index.add("db".to_string(), component);
    assert_eq!(index.components.len(), 1);

    index.clear();
    assert_eq!(index.components.len(), 0);
}

#[test]
fn test_index_manager_new() {
    let manager = IndexManager::new();

    // 验证索引为空
    assert_eq!(manager.find_symbol("test").len(), 0);
    assert!(manager.find_component("test").is_none());
    assert_eq!(manager.get_all_routes().len(), 0);
}

#[test]
fn test_index_manager_find_symbol() {
    let manager = IndexManager::new();

    // 初始状态应该找不到符号
    let symbols = manager.find_symbol("User");
    assert_eq!(symbols.len(), 0);
}

#[test]
fn test_index_manager_find_component() {
    let manager = IndexManager::new();

    // 初始状态应该找不到组件
    let component = manager.find_component("db");
    assert!(component.is_none());
}

#[test]
fn test_index_manager_get_all_routes() {
    let manager = IndexManager::new();

    // 初始状态应该没有路由
    let routes = manager.get_all_routes();
    assert_eq!(routes.len(), 0);
}

#[tokio::test]
async fn test_index_manager_build() {
    let manager = IndexManager::new();

    let workspace = Workspace {
        root_uri: Url::parse("file:///workspace").unwrap(),
        documents: vec![],
    };

    // 构建索引（异步操作）
    manager.build(&workspace).await;

    // 等待一小段时间让异步任务完成
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 验证索引已构建（虽然是空的）
    assert_eq!(manager.find_symbol("test").len(), 0);
}

#[test]
fn test_index_manager_update() {
    let manager = IndexManager::new();

    let uri = Url::parse("file:///test.rs").unwrap();
    let content = "struct User {}";

    // 更新索引
    manager.update(&uri, content);

    // 当前实现是 TODO，所以不会有实际效果
    // 这个测试主要验证方法可以被调用而不会崩溃
}

#[test]
fn test_symbol_type_equality() {
    assert_eq!(SymbolType::Struct, SymbolType::Struct);
    assert_eq!(SymbolType::Function, SymbolType::Function);
    assert_eq!(SymbolType::Const, SymbolType::Const);
    assert_eq!(SymbolType::Static, SymbolType::Static);
    assert_eq!(SymbolType::Module, SymbolType::Module);

    assert_ne!(SymbolType::Struct, SymbolType::Function);
    assert_ne!(SymbolType::Function, SymbolType::Const);
}

#[test]
fn test_symbol_info_clone() {
    let symbol = SymbolInfo {
        name: "User".to_string(),
        symbol_type: SymbolType::Struct,
        location: Location {
            uri: Url::parse("file:///test.rs").unwrap(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 20,
                    character: 0,
                },
            },
        },
    };

    let cloned = symbol.clone();
    assert_eq!(symbol.name, cloned.name);
    assert_eq!(symbol.symbol_type, cloned.symbol_type);
}

#[test]
fn test_component_info_clone() {
    let component = ComponentInfo {
        name: "db".to_string(),
        type_name: "ConnectPool".to_string(),
        location: Location {
            uri: Url::parse("file:///main.rs").unwrap(),
            range: Range {
                start: Position {
                    line: 15,
                    character: 0,
                },
                end: Position {
                    line: 20,
                    character: 0,
                },
            },
        },
        plugin: Some("spring-sqlx".to_string()),
    };

    let cloned = component.clone();
    assert_eq!(component.name, cloned.name);
    assert_eq!(component.type_name, cloned.type_name);
    assert_eq!(component.plugin, cloned.plugin);
}

// 并发访问测试
#[test]
fn test_symbol_index_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let index = Arc::new(SymbolIndex::new());
    let mut handles = vec![];

    // 启动多个线程同时添加符号
    for i in 0..10 {
        let index_clone = Arc::clone(&index);
        let handle = thread::spawn(move || {
            let symbol = SymbolInfo {
                name: format!("Symbol{}", i),
                symbol_type: SymbolType::Struct,
                location: Location {
                    uri: Url::parse("file:///test.rs").unwrap(),
                    range: Range {
                        start: Position {
                            line: i as u32,
                            character: 0,
                        },
                        end: Position {
                            line: i as u32 + 10,
                            character: 0,
                        },
                    },
                },
            };
            index_clone.add(format!("Symbol{}", i), symbol);
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 验证所有符号都已添加
    for i in 0..10 {
        let found = index.find(&format!("Symbol{}", i));
        assert_eq!(found.len(), 1);
    }
}

#[test]
fn test_component_index_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let index = Arc::new(ComponentIndex::new());
    let mut handles = vec![];

    // 启动多个线程同时添加组件
    for i in 0..10 {
        let index_clone = Arc::clone(&index);
        let handle = thread::spawn(move || {
            let component = ComponentInfo {
                name: format!("component{}", i),
                type_name: format!("Type{}", i),
                location: Location {
                    uri: Url::parse("file:///test.rs").unwrap(),
                    range: Range {
                        start: Position {
                            line: i as u32,
                            character: 0,
                        },
                        end: Position {
                            line: i as u32 + 10,
                            character: 0,
                        },
                    },
                },
                plugin: Some(format!("plugin{}", i)),
            };
            index_clone.add(format!("component{}", i), component);
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 验证所有组件都已添加
    for i in 0..10 {
        let found = index.find(&format!("component{}", i));
        assert!(found.is_some());
    }
}

#[test]
fn test_index_manager_concurrent_find() {
    use std::sync::Arc;
    use std::thread;

    let manager = Arc::new(IndexManager::new());
    let mut handles = vec![];

    // 启动多个线程同时查找
    for _ in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            // 查找符号
            let _symbols = manager_clone.find_symbol("test");
            // 查找组件
            let _component = manager_clone.find_component("test");
            // 获取路由
            let _routes = manager_clone.get_all_routes();
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }
}
