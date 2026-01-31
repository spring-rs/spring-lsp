//! 测试 taplo API 的使用

#[test]
fn test_taplo_table_api() {
    let toml_content = "[web]\nhost = \"localhost\"\nport = 8080";
    let root = taplo::parser::parse(toml_content).into_dom();
    
    // 尝试获取表
    if let Some(table) = root.as_table() {
        println!("Got table!");
        
        // 尝试获取 entries - 这是一个 Shared<Entries>
        let entries = table.entries();
        
        // Shared 类型需要通过 as_ref() 或 deref 来访问
        // 让我们尝试直接迭代
        for (key, value) in entries.iter() {
            println!("Key: {}, Value: {:?}", key.value(), value);
        }
    }
}

#[test]
fn test_taplo_integer_value() {
    let toml_content = "positive = 42\nnegative = -10";
    let root = taplo::parser::parse(toml_content).into_dom();
    
    if let Some(table) = root.as_table() {
        for (key, value) in table.entries().iter() {
            if let Some(int_node) = value.as_integer() {
                let int_val = int_node.value();
                println!("Key: {}, Integer value type: {:?}", key.value(), std::any::type_name_of_val(&int_val));
                
                // 尝试转换为 i64
                match int_val {
                    taplo::dom::node::IntegerValue::Positive(v) => println!("Positive: {}", v),
                    taplo::dom::node::IntegerValue::Negative(v) => println!("Negative: {}", v),
                }
            }
        }
    }
}
