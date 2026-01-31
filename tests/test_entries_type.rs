//! 测试 entries 的类型

#[test]
fn test_entries_type() {
    let toml_content = "[web]\nhost = \"localhost\"";
    let root = taplo::parser::parse(toml_content).into_dom();
    
    if let Some(table) = root.as_table() {
        let entries = table.entries();
        // 打印类型
        println!("Type: {}", std::any::type_name_of_val(&entries));
        
        // 尝试直接迭代
        for (key, _value) in entries.iter() {
            println!("Key: {}", key.value());
        }
    }
}
