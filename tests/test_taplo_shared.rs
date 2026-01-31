//! 测试 taplo Shared 类型的使用

#[test]
fn test_shared_deref() {
    let toml_content = "[web]\nhost = \"localhost\"";
    let root = taplo::parser::parse(toml_content).into_dom();
    
    if let Some(table) = root.as_table() {
        let entries = table.entries();
        
        // 尝试使用 Deref
        use std::ops::Deref;
        let entries_ref = entries.deref();
        
        for (key, value) in entries_ref.iter() {
            println!("Key: {}, Value: {:?}", key.value(), value);
        }
    }
}
