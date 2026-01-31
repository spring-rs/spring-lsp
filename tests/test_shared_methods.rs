//! 测试 Shared 类型的可用方法

#[test]
fn test_what_deref_returns() {
    let toml_content = "[web]\nhost = \"localhost\"";
    let root = taplo::parser::parse(toml_content).into_dom();
    
    if let Some(table) = root.as_table() {
        let entries = table.entries();
        
        // 尝试直接使用 * 操作符解引用
        for (key, _value) in (*entries).iter() {
            println!("Key: {}", key.value());
        }
    }
}
