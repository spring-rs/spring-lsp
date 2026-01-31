// 测试 Shared 类型的 Deref 实现

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
        
        // 克隆
        let entries_clone = entries_ref.clone();
        println!("Clone type: {}", std::any::type_name_of_val(&entries_clone));
        
        // 尝试解引用克隆的值
        println!("\n尝试解引用克隆的值:");
        for (k, v) in (&*entries_clone).iter() {
            println!("  Key: {}", k.value());
        }
    }
}
