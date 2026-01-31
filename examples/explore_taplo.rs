// 探索 taplo API 的正确使用方式

fn main() {
    let content = r#"
[web]
host = "localhost"
port = 8080

[redis]
url = "redis://localhost"
"#;

    println!("=== 解析 TOML ===");
    let parse_result = taplo::parser::parse(content);
    
    if !parse_result.errors.is_empty() {
        println!("解析错误:");
        for error in &parse_result.errors {
            println!("  {:?}", error);
        }
        return;
    }
    
    println!("解析成功！");
    
    let root = parse_result.into_dom();
    println!("Root 节点类型: {:?}", std::any::type_name_of_val(&root));
    
    // 尝试访问表
    if let Some(table) = root.as_table() {
        println!("\n=== 表信息 ===");
        println!("Table 类型: {}", std::any::type_name_of_val(&table));
        
        let entries = table.entries();
        println!("Entries 类型: {}", std::any::type_name_of_val(&entries));
        
        // 尝试不同的访问方法
        println!("\n=== 尝试访问 Entries ===");
        
        // 方法 1: 直接调用 iter()
        println!("尝试 entries.iter()...");
        // let iter = entries.iter(); // 这会失败
        
        // 方法 2: 使用 get() 获取 Arc 引用，然后迭代
        println!("尝试 entries.get().iter()...");
        let entries_arc = entries.get();
        for (key, value) in entries_arc.iter() {
            println!("  Key: {}", key.value());
            println!("  Value type: {:?}", std::any::type_name_of_val(&value));
        }
        
        println!("\n成功！使用 entries.get().iter() 可以访问条目");
    }
}
