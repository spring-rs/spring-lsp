//! 测试 taplo Shared 类型的正确使用方法

use std::ops::Deref;

#[test]
fn test_shared_deref() {
    let toml_content = "[web]\nhost = \"localhost\"";
    let root = taplo::parser::parse(toml_content).into_dom();
    
    if let Some(table) = root.as_table() {
        let entries = table.entries();
        
        // Shared 实现了 Deref，所以我们可以通过 deref 访问内部值
        // 然后调用 iter()
        for (key, _value) in entries.deref().iter() {
            println!("Key: {}", key.value());
        }
    }
}

#[test]
fn test_array_items() {
    let toml_content = "items = [1, 2, 3]";
    let root = taplo::parser::parse(toml_content).into_dom();
    
    if let Some(table) = root.as_table() {
        let entries = table.entries();
        for (_key, value) in entries.deref().iter() {
            if let Some(arr) = value.as_array() {
                let items = arr.items();
                for item in items.deref().iter() {
                    println!("Item: {:?}", item);
                }
            }
        }
    }
}
