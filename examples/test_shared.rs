// 测试 Shared 类型的访问方法

fn main() {
    let content = r#"
[web]
host = "localhost"
"#;

    let parse_result = taplo::parser::parse(content);
    let root = parse_result.into_dom();
    
    if let Some(table) = root.as_table() {
        let entries_ref = table.entries();
        
        println!("Entries ref type: {}", std::any::type_name_of_val(&entries_ref));
        
        // 尝试不同的方法
        println!("\n尝试方法 1: 直接调用 iter()");
        // for (k, v) in entries_ref.iter() {} // 失败
        
        println!("尝试方法 2: 解引用后调用 iter()");
        // for (k, v) in (*entries_ref).iter() {} // 失败
        
        println!("尝试方法 3: 使用 as_ref()");
        // if let Some(entries) = entries_ref.as_ref() {} // 失败
        
        println!("尝试方法 4: 克隆");
        let entries_clone = entries_ref.clone();
        println!("Clone type: {}", std::any::type_name_of_val(&entries_clone));
        for (k, v) in entries_clone.iter() {
            println!("  Key: {}", k.value());
        }
        
        println!("\n成功！使用 clone() 然后调用 iter()");
    }
}
