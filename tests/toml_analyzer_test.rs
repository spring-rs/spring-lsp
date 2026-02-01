//! TomlAnalyzer 单元测试

use spring_lsp::schema::SchemaProvider;
use spring_lsp::toml_analyzer::{ConfigValue, TomlAnalyzer};

fn create_analyzer() -> TomlAnalyzer {
    let schema_provider = SchemaProvider::default();
    TomlAnalyzer::new(schema_provider)
}

#[test]
fn test_parse_empty_toml() {
    let analyzer = create_analyzer();
    let result = analyzer.parse("");
    assert!(result.is_ok());
    let doc = result.unwrap();
    assert_eq!(doc.config_sections.len(), 0);
    assert_eq!(doc.env_vars.len(), 0);
}

#[test]
fn test_parse_invalid_toml() {
    let analyzer = create_analyzer();
    let result = analyzer.parse("[invalid");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("TOML 语法错误"));
}

#[test]
fn test_parse_simple_config() {
    let analyzer = create_analyzer();
    let toml = r#"
[web]
host = "localhost"
port = 8080
"#;

    let result = analyzer.parse(toml);
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.config_sections.len(), 1);

    let web_section = doc.config_sections.get("web").unwrap();
    assert_eq!(web_section.prefix, "web");
    assert_eq!(web_section.properties.len(), 2);

    let host = web_section.properties.get("host").unwrap();
    assert_eq!(host.key, "host");
    assert_eq!(host.value, ConfigValue::String("localhost".to_string()));

    let port = web_section.properties.get("port").unwrap();
    assert_eq!(port.key, "port");
    assert_eq!(port.value, ConfigValue::Integer(8080));
}

#[test]
fn test_env_var_with_default() {
    let analyzer = create_analyzer();
    let toml = r#"
[web]
host = "${HOST:localhost}"
"#;

    let result = analyzer.parse(toml);
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.env_vars.len(), 1);

    let env_var = &doc.env_vars[0];
    assert_eq!(env_var.name, "HOST");
    assert_eq!(env_var.default, Some("localhost".to_string()));
}

#[test]
fn test_env_var_without_default() {
    let analyzer = create_analyzer();
    let toml = r#"
[web]
host = "${HOST}"
"#;

    let result = analyzer.parse(toml);
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.env_vars.len(), 1);

    let env_var = &doc.env_vars[0];
    assert_eq!(env_var.name, "HOST");
    assert_eq!(env_var.default, None);
}

#[test]
fn test_multiple_config_sections() {
    let analyzer = create_analyzer();
    let toml = r#"
[web]
host = "localhost"

[redis]
url = "redis://localhost"
"#;

    let result = analyzer.parse(toml);
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.config_sections.len(), 2);
    assert!(doc.config_sections.contains_key("web"));
    assert!(doc.config_sections.contains_key("redis"));
}

#[test]
fn test_array_value() {
    let analyzer = create_analyzer();
    let toml = r#"
[web]
allowed_origins = ["http://localhost:3000", "http://localhost:8080"]
"#;

    let result = analyzer.parse(toml);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let web_section = doc.config_sections.get("web").unwrap();
    let allowed_origins = web_section.properties.get("allowed_origins").unwrap();

    match &allowed_origins.value {
        ConfigValue::Array(items) => {
            assert_eq!(items.len(), 2);
            assert_eq!(
                items[0],
                ConfigValue::String("http://localhost:3000".to_string())
            );
            assert_eq!(
                items[1],
                ConfigValue::String("http://localhost:8080".to_string())
            );
        }
        _ => panic!("Expected array value"),
    }
}

#[test]
fn test_boolean_value() {
    let analyzer = create_analyzer();
    let toml = r#"
[web]
enabled = true
debug = false
"#;

    let result = analyzer.parse(toml);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let web_section = doc.config_sections.get("web").unwrap();

    let enabled = web_section.properties.get("enabled").unwrap();
    assert_eq!(enabled.value, ConfigValue::Boolean(true));

    let debug = web_section.properties.get("debug").unwrap();
    assert_eq!(debug.value, ConfigValue::Boolean(false));
}

#[test]
fn test_float_value() {
    let analyzer = create_analyzer();
    let toml = r#"
[web]
timeout = 30.5
"#;

    let result = analyzer.parse(toml);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let web_section = doc.config_sections.get("web").unwrap();
    let timeout = web_section.properties.get("timeout").unwrap();
    assert_eq!(timeout.value, ConfigValue::Float(30.5));
}

#[test]
fn test_multi_environment_config() {
    let analyzer = create_analyzer();

    // 测试 app-dev.toml
    let dev_toml = r#"
[web]
host = "localhost"
port = 8080
"#;

    let result = analyzer.parse(dev_toml);
    assert!(result.is_ok());

    // 测试 app-prod.toml
    let prod_toml = r#"
[web]
host = "0.0.0.0"
port = 80
"#;

    let result = analyzer.parse(prod_toml);
    assert!(result.is_ok());
}

#[test]
fn test_nested_table_value() {
    let analyzer = create_analyzer();
    let toml = r#"
[database]
[database.connection]
host = "localhost"
port = 5432
"#;

    let result = analyzer.parse(toml);
    assert!(result.is_ok());

    let doc = result.unwrap();
    // 应该识别出 database 和 database.connection 两个节
    assert!(doc.config_sections.contains_key("database"));
}

#[test]
fn test_multiple_env_vars_in_line() {
    let analyzer = create_analyzer();
    let toml = r#"
[web]
url = "${PROTOCOL:http}://${HOST:localhost}:${PORT:8080}"
"#;

    let result = analyzer.parse(toml);
    assert!(result.is_ok());

    let doc = result.unwrap();
    // 应该识别出 3 个环境变量
    assert_eq!(doc.env_vars.len(), 3);
    assert_eq!(doc.env_vars[0].name, "PROTOCOL");
    assert_eq!(doc.env_vars[0].default, Some("http".to_string()));
    assert_eq!(doc.env_vars[1].name, "HOST");
    assert_eq!(doc.env_vars[1].default, Some("localhost".to_string()));
    assert_eq!(doc.env_vars[2].name, "PORT");
    assert_eq!(doc.env_vars[2].default, Some("8080".to_string()));
}

#[test]
fn test_env_var_in_array() {
    let analyzer = create_analyzer();
    let toml = r#"
[web]
hosts = ["${HOST1:localhost}", "${HOST2:127.0.0.1}"]
"#;

    let result = analyzer.parse(toml);
    assert!(result.is_ok());

    let doc = result.unwrap();
    // 应该识别出 2 个环境变量
    assert_eq!(doc.env_vars.len(), 2);
    assert_eq!(doc.env_vars[0].name, "HOST1");
    assert_eq!(doc.env_vars[1].name, "HOST2");
}

#[test]
fn test_negative_integer() {
    let analyzer = create_analyzer();
    let toml = r#"
[web]
offset = -100
temperature = -273
"#;

    let result = analyzer.parse(toml);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let web_section = doc.config_sections.get("web").unwrap();

    let offset = web_section.properties.get("offset").unwrap();
    assert_eq!(offset.value, ConfigValue::Integer(-100));

    let temperature = web_section.properties.get("temperature").unwrap();
    assert_eq!(temperature.value, ConfigValue::Integer(-273));
}

#[test]
fn test_empty_config_section() {
    let analyzer = create_analyzer();
    let toml = r#"
[web]
"#;

    let result = analyzer.parse(toml);
    assert!(result.is_ok());

    let doc = result.unwrap();
    assert_eq!(doc.config_sections.len(), 1);

    let web_section = doc.config_sections.get("web").unwrap();
    assert_eq!(web_section.properties.len(), 0);
}

#[test]
fn test_syntax_error_unclosed_bracket() {
    let analyzer = create_analyzer();
    let result = analyzer.parse("[web");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("TOML 语法错误"));
}

#[test]
fn test_syntax_error_invalid_value() {
    let analyzer = create_analyzer();
    let toml = r#"
[web]
port = not_a_number
"#;

    let result = analyzer.parse(toml);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("TOML 语法错误"));
}

#[test]
fn test_empty_string_value() {
    let analyzer = create_analyzer();
    let toml = r#"
[web]
host = ""
"#;

    let result = analyzer.parse(toml);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let web_section = doc.config_sections.get("web").unwrap();
    let host = web_section.properties.get("host").unwrap();
    assert_eq!(host.value, ConfigValue::String("".to_string()));
}

#[test]
fn test_empty_array() {
    let analyzer = create_analyzer();
    let toml = r#"
[web]
allowed_origins = []
"#;

    let result = analyzer.parse(toml);
    assert!(result.is_ok());

    let doc = result.unwrap();
    let web_section = doc.config_sections.get("web").unwrap();
    let allowed_origins = web_section.properties.get("allowed_origins").unwrap();

    match &allowed_origins.value {
        ConfigValue::Array(items) => {
            assert_eq!(items.len(), 0);
        }
        _ => panic!("Expected array value"),
    }
}
