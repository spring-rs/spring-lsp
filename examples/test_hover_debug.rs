use lsp_types::Position;
use spring_lsp::schema::{
    ConfigSchema, PluginSchema, PropertySchema, SchemaProvider, TypeInfo, Value,
};
use spring_lsp::toml_analyzer::TomlAnalyzer;
use std::collections::HashMap;

fn create_test_schema_provider() -> SchemaProvider {
    let mut plugins = HashMap::new();

    let mut web_properties = HashMap::new();
    web_properties.insert(
        "host".to_string(),
        PropertySchema {
            name: "host".to_string(),
            type_info: TypeInfo::String {
                enum_values: None,
                min_length: Some(1),
                max_length: Some(255),
            },
            description: "Web 服务器监听地址".to_string(),
            default: Some(Value::String("localhost".to_string())),
            required: false,
            deprecated: None,
            example: Some("localhost".to_string()),
        },
    );

    plugins.insert(
        "web".to_string(),
        PluginSchema {
            prefix: "web".to_string(),
            properties: web_properties,
        },
    );

    let schema = ConfigSchema { plugins };
    SchemaProvider::from_schema(schema)
}

fn main() {
    let schema_provider = create_test_schema_provider();
    let analyzer = TomlAnalyzer::new(schema_provider);

    let toml_content = r#"
[web]
host = "localhost"
port = 8080
"#;

    let doc = analyzer.parse(toml_content).unwrap();

    println!("Config sections: {:?}", doc.config_sections.keys());

    for (prefix, section) in &doc.config_sections {
        println!("\nSection: {}", prefix);
        println!("  Range: {:?}", section.range);
        for (key, property) in &section.properties {
            println!("  Property: {}", key);
            println!("    Range: {:?}", property.range);
            println!("    Value: {:?}", property.value);
        }
    }

    // Test hover at different positions
    let positions = vec![
        Position {
            line: 0,
            character: 0,
        },
        Position {
            line: 1,
            character: 0,
        },
        Position {
            line: 2,
            character: 0,
        },
        Position {
            line: 2,
            character: 5,
        },
        Position {
            line: 2,
            character: 10,
        },
    ];

    for pos in positions {
        println!("\nTesting position: {:?}", pos);
        let hover = analyzer.hover(&doc, pos);
        println!(
            "  Hover result: {}",
            if hover.is_some() { "Some" } else { "None" }
        );
    }
}
