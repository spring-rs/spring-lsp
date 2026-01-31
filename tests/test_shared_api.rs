//! 测试 Shared API

#[test]
fn test_shared_get() {
    let toml_content = "[web]\nhost = \"localhost\"";
    let root = taplo::parser::parse(toml_content).into_dom();
    
    if let Some(table) = root.as_table() {
        let entries = table.entries();
        
        // 尝试使用 get() 方法
        if let Some(entries_arc) = entries.get() {
            for (key, _value) in entries_arc.iter() {
                println!("Key: {}", key.value());
            }
        }
    }
}
