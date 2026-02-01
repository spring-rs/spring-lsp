// 测试 taplo API 的使用方式

fn main() {
    let content = r#"
[web]
host = "localhost"
port = 8080

[redis]
url = "redis://localhost"
"#;

    let parse_result = taplo::parser::parse(content);
    let root = parse_result.into_dom();

    println!("Root node type: {:?}", std::any::type_name_of_val(&root));

    if let Some(table) = root.as_table() {
        println!("Table type: {:?}", std::any::type_name_of_val(&table));
        println!(
            "Entries type: {:?}",
            std::any::type_name_of_val(&table.entries())
        );

        // 尝试不同的方法访问 entries
        let entries = table.entries();
        println!("Entries: {:?}", entries);

        // 尝试直接迭代
        // for (key, value) in entries {
        //     println!("Key: {}, Value: {:?}", key, value);
        // }
    }
}
