//! 路由导航模块
//!
//! 提供路由识别、索引和导航功能

use crate::macro_analyzer::HttpMethod;
use lsp_types::Location;
use std::collections::HashMap;

/// 路由导航器
///
/// 负责识别、索引和导航项目中的所有路由
#[derive(Debug, Clone)]
pub struct RouteNavigator {
    /// 路由索引
    pub index: RouteIndex,
}

impl RouteNavigator {
    /// 创建新的路由导航器
    pub fn new() -> Self {
        Self {
            index: RouteIndex::new(),
        }
    }

    /// 构建路由索引
    ///
    /// 从 Rust 文档列表中提取所有路由信息，构建路由索引
    ///
    /// # Arguments
    ///
    /// * `documents` - Rust 文档列表，包含已解析的宏信息
    ///
    /// # Requirements
    ///
    /// - 8.1: 识别所有路由宏标注的函数
    /// - 8.2: 解析路径参数
    /// - 8.3: 处理多方法路由
    /// - 8.4: 解析路由前缀
    pub fn build_index(&mut self, documents: &[crate::macro_analyzer::RustDocument]) {
        // 清空现有索引
        self.index.routes.clear();
        self.index.path_map.clear();

        // 遍历所有文档
        for doc in documents {
            // 遍历文档中的所有宏
            for spring_macro in &doc.macros {
                // 只处理路由宏
                if let crate::macro_analyzer::SpringMacro::Route(route_macro) = spring_macro {
                    // 为每个 HTTP 方法创建独立的路由条目（需求 8.3）
                    for method in &route_macro.methods {
                        // 解析路径参数（需求 8.2）
                        let path_params = self.parse_path_parameters(&route_macro.path);

                        // 提取处理器函数参数
                        // 注意：当前实现中，我们从 route_macro 中没有函数参数信息
                        // 这需要在 MacroAnalyzer 中扩展以提取函数签名
                        let parameters = path_params
                            .iter()
                            .map(|param_name| Parameter {
                                name: param_name.clone(),
                                type_name: "Unknown".to_string(), // 需要从函数签名中提取
                            })
                            .collect();

                        // 创建路由信息
                        let route_info = RouteInfo {
                            path: route_macro.path.clone(),
                            methods: vec![method.clone()],
                            handler: HandlerInfo {
                                function_name: route_macro.handler_name.clone(),
                                parameters,
                            },
                            location: Location {
                                uri: doc.uri.clone(),
                                range: route_macro.range,
                            },
                        };

                        // 添加到索引
                        let route_index = self.index.routes.len();
                        self.index.routes.push(route_info);

                        // 更新路径映射
                        self.index
                            .path_map
                            .entry(route_macro.path.clone())
                            .or_default()
                            .push(route_index);
                    }
                }
            }
        }
    }

    /// 查找路由
    ///
    /// 支持模糊匹配和正则表达式搜索路由
    ///
    /// # Arguments
    ///
    /// * `pattern` - 搜索模式，可以是：
    ///   - 普通字符串：进行模糊匹配（路径包含该字符串）
    ///   - 正则表达式：以 "regex:" 开头，如 "regex:^/api/.*"
    ///
    /// # Returns
    ///
    /// 返回匹配的路由信息列表
    ///
    /// # Requirements
    ///
    /// - 9.4: 支持模糊匹配和正则表达式搜索
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_lsp::route::RouteNavigator;
    ///
    /// let navigator = RouteNavigator::new();
    /// // 模糊匹配
    /// let routes = navigator.find_routes("users");
    /// // 正则表达式匹配
    /// let routes = navigator.find_routes("regex:^/api/v[0-9]+/.*");
    /// ```
    pub fn find_routes(&self, pattern: &str) -> Vec<&RouteInfo> {
        if pattern.is_empty() {
            return Vec::new();
        }

        // 检查是否是正则表达式模式
        if let Some(regex_pattern) = pattern.strip_prefix("regex:") {
            // 使用正则表达式匹配
            if let Ok(re) = regex::Regex::new(regex_pattern) {
                self.index
                    .routes
                    .iter()
                    .filter(|route| re.is_match(&route.path))
                    .collect()
            } else {
                // 正则表达式无效，返回空列表
                Vec::new()
            }
        } else {
            // 使用模糊匹配（路径包含搜索字符串）
            self.index
                .routes
                .iter()
                .filter(|route| route.path.contains(pattern))
                .collect()
        }
    }

    /// 获取所有路由
    ///
    /// 返回项目中所有已识别的路由
    ///
    /// # Returns
    ///
    /// 返回所有路由信息的引用
    ///
    /// # Requirements
    ///
    /// - 9.1: 返回项目中所有路由的列表
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_lsp::route::RouteNavigator;
    ///
    /// let navigator = RouteNavigator::new();
    /// let all_routes = navigator.get_all_routes();
    /// println!("Total routes: {}", all_routes.len());
    /// ```
    pub fn get_all_routes(&self) -> &[RouteInfo] {
        &self.index.routes
    }

    /// 根据处理器函数名查找路由
    ///
    /// 实现处理器路由反查功能，根据处理器函数名查找对应的所有路由
    ///
    /// # Arguments
    ///
    /// * `handler_name` - 处理器函数名称
    ///
    /// # Returns
    ///
    /// 返回该处理器对应的所有路由信息
    ///
    /// # Requirements
    ///
    /// - 9.3: 实现处理器路由反查
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_lsp::route::RouteNavigator;
    ///
    /// let navigator = RouteNavigator::new();
    /// let routes = navigator.find_routes_by_handler("get_user");
    /// for route in routes {
    ///     println!("Handler 'get_user' handles: {:?} {}", route.methods[0], route.path);
    /// }
    /// ```
    pub fn find_routes_by_handler(&self, handler_name: &str) -> Vec<&RouteInfo> {
        self.index
            .routes
            .iter()
            .filter(|route| route.handler.function_name == handler_name)
            .collect()
    }

    /// 验证路由
    ///
    /// 验证所有路由的正确性，包括：
    /// - 路径字符验证
    /// - 路径参数语法验证
    /// - 路径参数类型匹配验证
    /// - RESTful 风格检查
    ///
    /// # Returns
    ///
    /// 返回诊断信息列表
    ///
    /// # Requirements
    ///
    /// - 10.1: 路径字符验证
    /// - 10.2: 路径参数语法验证
    /// - 10.3: 路径参数类型匹配验证
    /// - 10.5: RESTful 风格检查
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_lsp::route::RouteNavigator;
    ///
    /// let navigator = RouteNavigator::new();
    /// let diagnostics = navigator.validate_routes();
    /// for diagnostic in diagnostics {
    ///     println!("Validation error: {}", diagnostic.message);
    /// }
    /// ```
    pub fn validate_routes(&self) -> Vec<lsp_types::Diagnostic> {
        let mut diagnostics = Vec::new();

        for route in &self.index.routes {
            // 验证路径字符（需求 10.1）
            diagnostics.extend(self.validate_path_characters(&route.path, &route.location));

            // 验证路径参数语法（需求 10.2）
            diagnostics.extend(self.validate_path_parameter_syntax(&route.path, &route.location));

            // 验证路径参数类型匹配（需求 10.3）
            diagnostics.extend(self.validate_path_parameter_types(route));

            // RESTful 风格检查（需求 10.5）
            diagnostics.extend(self.validate_restful_style(route));
        }

        diagnostics
    }

    /// 检测路由冲突
    ///
    /// 检测具有相同路径和 HTTP 方法的路由冲突
    ///
    /// # Returns
    ///
    /// 返回路由冲突列表
    ///
    /// # Requirements
    ///
    /// - 9.5: 检测路由冲突
    /// - 10.4: 路由路径冲突检测
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_lsp::route::RouteNavigator;
    ///
    /// let navigator = RouteNavigator::new();
    /// let conflicts = navigator.detect_conflicts();
    /// for conflict in conflicts {
    ///     println!("Route conflict detected between routes {} and {}",
    ///              conflict.index1, conflict.index2);
    /// }
    /// ```
    pub fn detect_conflicts(&self) -> Vec<RouteConflict> {
        let mut conflicts = Vec::new();

        // 遍历所有路由对，检测冲突
        for i in 0..self.index.routes.len() {
            for j in (i + 1)..self.index.routes.len() {
                let route1 = &self.index.routes[i];
                let route2 = &self.index.routes[j];

                // 检查路径是否相同
                if route1.path == route2.path {
                    // 检查是否有相同的 HTTP 方法
                    for method1 in &route1.methods {
                        if route2.methods.contains(method1) {
                            conflicts.push(RouteConflict {
                                index1: i,
                                index2: j,
                                path: route1.path.clone(),
                                method: method1.clone(),
                                location1: route1.location.clone(),
                                location2: route2.location.clone(),
                            });
                        }
                    }
                }
            }
        }

        conflicts
    }

    /// 验证路径字符
    ///
    /// 检查路径是否包含 URL 规范不允许的字符
    ///
    /// # Requirements
    ///
    /// - 10.1: 路径字符验证
    fn validate_path_characters(
        &self,
        path: &str,
        location: &Location,
    ) -> Vec<lsp_types::Diagnostic> {
        let mut diagnostics = Vec::new();

        // URL 路径允许的字符：字母、数字、-_.~:/?#[]@!$&'()*+,;=
        // 以及路径参数的 {}
        for (i, ch) in path.chars().enumerate() {
            if !ch.is_ascii_alphanumeric()
                && !matches!(
                    ch,
                    '-' | '_'
                        | '.'
                        | '~'
                        | ':'
                        | '/'
                        | '?'
                        | '#'
                        | '['
                        | ']'
                        | '@'
                        | '!'
                        | '$'
                        | '&'
                        | '\''
                        | '('
                        | ')'
                        | '*'
                        | '+'
                        | ','
                        | ';'
                        | '='
                        | '{'
                        | '}'
                )
            {
                diagnostics.push(lsp_types::Diagnostic {
                    range: location.range,
                    severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                    code: Some(lsp_types::NumberOrString::String("invalid-path-char".to_string())),
                    message: format!(
                        "路径包含无效字符 '{}' (位置 {})。URL 路径只能包含字母、数字和特定的特殊字符。",
                        ch, i
                    ),
                    source: Some("spring-lsp".to_string()),
                    ..Default::default()
                });
            }
        }

        diagnostics
    }

    /// 验证路径参数语法
    ///
    /// 检查路径参数是否符合 {param} 格式
    ///
    /// # Requirements
    ///
    /// - 10.2: 路径参数语法验证
    fn validate_path_parameter_syntax(
        &self,
        path: &str,
        location: &Location,
    ) -> Vec<lsp_types::Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut brace_count = 0;
        let mut last_open_brace = None;

        for (i, ch) in path.chars().enumerate() {
            match ch {
                '{' => {
                    if brace_count > 0 {
                        // 嵌套的大括号
                        diagnostics.push(lsp_types::Diagnostic {
                            range: location.range,
                            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                            code: Some(lsp_types::NumberOrString::String(
                                "nested-path-param".to_string(),
                            )),
                            message: format!("路径参数不能嵌套 (位置 {})。正确格式：{{param}}", i),
                            source: Some("spring-lsp".to_string()),
                            ..Default::default()
                        });
                    }
                    brace_count += 1;
                    last_open_brace = Some(i);
                }
                '}' => {
                    if brace_count == 0 {
                        // 没有匹配的开括号
                        diagnostics.push(lsp_types::Diagnostic {
                            range: location.range,
                            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                            code: Some(lsp_types::NumberOrString::String(
                                "unmatched-closing-brace".to_string(),
                            )),
                            message: format!(
                                "路径参数缺少开括号 '{{' (位置 {})。正确格式：{{param}}",
                                i
                            ),
                            source: Some("spring-lsp".to_string()),
                            ..Default::default()
                        });
                    } else {
                        // 检查参数名是否为空
                        if let Some(open_pos) = last_open_brace {
                            if i == open_pos + 1 {
                                diagnostics.push(lsp_types::Diagnostic {
                                    range: location.range,
                                    severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                                    code: Some(lsp_types::NumberOrString::String(
                                        "empty-path-param".to_string(),
                                    )),
                                    message: format!(
                                        "路径参数名称不能为空 (位置 {})。正确格式：{{param}}",
                                        open_pos
                                    ),
                                    source: Some("spring-lsp".to_string()),
                                    ..Default::default()
                                });
                            }
                        }
                        brace_count -= 1;
                    }
                }
                _ => {}
            }
        }

        // 检查是否有未闭合的括号
        if brace_count > 0 {
            if let Some(open_pos) = last_open_brace {
                diagnostics.push(lsp_types::Diagnostic {
                    range: location.range,
                    severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                    code: Some(lsp_types::NumberOrString::String(
                        "unclosed-path-param".to_string(),
                    )),
                    message: format!(
                        "路径参数缺少闭括号 '}}' (位置 {})。正确格式：{{param}}",
                        open_pos
                    ),
                    source: Some("spring-lsp".to_string()),
                    ..Default::default()
                });
            }
        }

        diagnostics
    }

    /// 验证路径参数类型匹配
    ///
    /// 检查路径参数类型与处理器函数参数类型是否兼容
    ///
    /// # Requirements
    ///
    /// - 10.3: 路径参数类型匹配验证
    fn validate_path_parameter_types(&self, route: &RouteInfo) -> Vec<lsp_types::Diagnostic> {
        let mut diagnostics = Vec::new();

        // 提取路径参数
        let path_params = self.parse_path_parameters(&route.path);

        // 检查每个路径参数是否在处理器参数中有对应的类型
        for path_param in &path_params {
            // 查找处理器参数中是否有匹配的参数
            let found = route
                .handler
                .parameters
                .iter()
                .any(|p| p.name == *path_param);

            if !found {
                diagnostics.push(lsp_types::Diagnostic {
                    range: route.location.range,
                    severity: Some(lsp_types::DiagnosticSeverity::WARNING),
                    code: Some(lsp_types::NumberOrString::String(
                        "missing-path-param".to_string(),
                    )),
                    message: format!(
                        "路径参数 '{}' 在处理器函数 '{}' 的参数列表中未找到。\
                         请确保函数参数中包含 Path<T> 类型的参数来接收此路径参数。",
                        path_param, route.handler.function_name
                    ),
                    source: Some("spring-lsp".to_string()),
                    ..Default::default()
                });
            }
        }

        // 检查类型兼容性（如果类型信息可用）
        for param in &route.handler.parameters {
            if path_params.contains(&param.name) {
                // 检查参数类型是否是 Path<T> 或兼容类型
                if !param.type_name.contains("Path")
                    && !param.type_name.contains("Unknown")
                    && !param.type_name.is_empty()
                {
                    diagnostics.push(lsp_types::Diagnostic {
                        range: route.location.range,
                        severity: Some(lsp_types::DiagnosticSeverity::WARNING),
                        code: Some(lsp_types::NumberOrString::String(
                            "incompatible-path-param-type".to_string(),
                        )),
                        message: format!(
                            "路径参数 '{}' 的类型 '{}' 可能不兼容。\
                             路径参数通常应该使用 Path<T> 类型。",
                            param.name, param.type_name
                        ),
                        source: Some("spring-lsp".to_string()),
                        ..Default::default()
                    });
                }
            }
        }

        diagnostics
    }

    /// 验证 RESTful 风格
    ///
    /// 检查路由路径是否符合 RESTful 命名规范
    ///
    /// # Requirements
    ///
    /// - 10.5: RESTful 风格检查
    fn validate_restful_style(&self, route: &RouteInfo) -> Vec<lsp_types::Diagnostic> {
        let mut diagnostics = Vec::new();

        // 分割路径为段
        let segments: Vec<&str> = route.path.split('/').filter(|s| !s.is_empty()).collect();

        for segment in &segments {
            // 跳过路径参数
            if segment.starts_with('{') && segment.ends_with('}') {
                continue;
            }

            // 检查是否使用了动词（RESTful 应该使用名词）
            let verbs = [
                "get", "post", "put", "delete", "patch", "create", "update", "remove", "add",
                "list", "fetch", "retrieve", "save", "destroy",
            ];

            let segment_lower = segment.to_lowercase();
            for verb in &verbs {
                // 只匹配完整的单词或以动词开头的驼峰命名/连字符命名
                // 例如：匹配 "get", "getUsers", "get-users"
                // 但不匹配 "posts" (虽然包含 "post")
                let is_verb_match = if segment_lower == *verb {
                    // 完全匹配
                    true
                } else if segment_lower.starts_with(verb) && segment_lower.len() > verb.len() {
                    // 检查动词后面的字符
                    let next_char = segment.chars().nth(verb.len()).unwrap();
                    // 驼峰命名（getUsers）或连字符/下划线命名（get-users, get_users）
                    next_char.is_uppercase() || next_char == '-' || next_char == '_'
                } else {
                    false
                };

                if is_verb_match {
                    diagnostics.push(lsp_types::Diagnostic {
                        range: route.location.range,
                        severity: Some(lsp_types::DiagnosticSeverity::INFORMATION),
                        code: Some(lsp_types::NumberOrString::String(
                            "restful-style-verb".to_string(),
                        )),
                        message: format!(
                            "路径段 '{}' 包含动词 '{}'。RESTful API 建议使用名词而非动词，\
                             通过 HTTP 方法（GET、POST、PUT、DELETE）来表示操作。\
                             例如：使用 'GET /users' 而非 'GET /getUsers'。",
                            segment, verb
                        ),
                        source: Some("spring-lsp".to_string()),
                        ..Default::default()
                    });
                    break;
                }
            }

            // 检查是否使用了驼峰命名（RESTful 建议使用小写和连字符）
            if segment.chars().any(|c| c.is_uppercase()) {
                diagnostics.push(lsp_types::Diagnostic {
                    range: route.location.range,
                    severity: Some(lsp_types::DiagnosticSeverity::INFORMATION),
                    code: Some(lsp_types::NumberOrString::String(
                        "restful-style-case".to_string(),
                    )),
                    message: format!(
                        "路径段 '{}' 使用了大写字母。RESTful API 建议使用小写字母和连字符。\
                         例如：使用 '/user-profiles' 而非 '/userProfiles'。",
                        segment
                    ),
                    source: Some("spring-lsp".to_string()),
                    ..Default::default()
                });
            }
        }

        diagnostics
    }

    /// 解析路径参数
    ///
    /// 从路由路径中提取所有参数名称（如 "/users/{id}" 中的 "id"）
    ///
    /// # Arguments
    ///
    /// * `path` - 路由路径
    ///
    /// # Returns
    ///
    /// 返回参数名称列表
    ///
    /// # Requirements
    ///
    /// - 8.2: 正确解析路径参数
    fn parse_path_parameters(&self, path: &str) -> Vec<String> {
        let mut parameters = Vec::new();
        let mut in_param = false;
        let mut param_start = 0;

        for (i, ch) in path.char_indices() {
            match ch {
                '{' => {
                    in_param = true;
                    param_start = i + 1;
                }
                '}' => {
                    if in_param {
                        let param_name = &path[param_start..i];
                        if !param_name.is_empty() {
                            parameters.push(param_name.to_string());
                        }
                        in_param = false;
                    }
                }
                _ => {}
            }
        }

        parameters
    }
}

impl Default for RouteNavigator {
    fn default() -> Self {
        Self::new()
    }
}

/// 路由索引
///
/// 存储所有路由信息，提供快速查找功能
#[derive(Debug, Clone)]
pub struct RouteIndex {
    /// 所有路由列表
    pub routes: Vec<RouteInfo>,
    /// 路径到路由索引的映射（用于快速查找）
    /// Key: 路由路径, Value: routes 数组中的索引列表
    pub path_map: HashMap<String, Vec<usize>>,
}

impl RouteIndex {
    /// 创建新的路由索引
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            path_map: HashMap::new(),
        }
    }
}

impl Default for RouteIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// 路由信息
///
/// 描述单个路由的完整信息
#[derive(Debug, Clone)]
pub struct RouteInfo {
    /// 路由路径（如 "/users/{id}"）
    pub path: String,
    /// HTTP 方法列表
    pub methods: Vec<HttpMethod>,
    /// 处理器函数信息
    pub handler: HandlerInfo,
    /// 路由在源代码中的位置
    pub location: Location,
}

/// 处理器函数信息
///
/// 描述路由处理器函数的详细信息
#[derive(Debug, Clone)]
pub struct HandlerInfo {
    /// 函数名称
    pub function_name: String,
    /// 函数参数列表
    pub parameters: Vec<Parameter>,
}

/// 函数参数信息
///
/// 描述处理器函数的单个参数
#[derive(Debug, Clone)]
pub struct Parameter {
    /// 参数名称
    pub name: String,
    /// 参数类型
    pub type_name: String,
}

/// 路由冲突信息
///
/// 描述两个路由之间的冲突
#[derive(Debug, Clone)]
pub struct RouteConflict {
    /// 第一个路由的索引
    pub index1: usize,
    /// 第二个路由的索引
    pub index2: usize,
    /// 冲突的路径
    pub path: String,
    /// 冲突的 HTTP 方法
    pub method: HttpMethod,
    /// 第一个路由的位置
    pub location1: Location,
    /// 第二个路由的位置
    pub location2: Location,
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{Position, Range, Url};

    #[test]
    fn test_route_navigator_new() {
        let navigator = RouteNavigator::new();
        assert_eq!(navigator.index.routes.len(), 0);
        assert_eq!(navigator.index.path_map.len(), 0);
    }

    #[test]
    fn test_route_navigator_default() {
        let navigator = RouteNavigator::default();
        assert_eq!(navigator.index.routes.len(), 0);
        assert_eq!(navigator.index.path_map.len(), 0);
    }

    #[test]
    fn test_route_index_new() {
        let index = RouteIndex::new();
        assert_eq!(index.routes.len(), 0);
        assert_eq!(index.path_map.len(), 0);
    }

    #[test]
    fn test_route_index_default() {
        let index = RouteIndex::default();
        assert_eq!(index.routes.len(), 0);
        assert_eq!(index.path_map.len(), 0);
    }

    #[test]
    fn test_route_info_creation() {
        let route = RouteInfo {
            path: "/users/{id}".to_string(),
            methods: vec![HttpMethod::Get],
            handler: HandlerInfo {
                function_name: "get_user".to_string(),
                parameters: vec![Parameter {
                    name: "id".to_string(),
                    type_name: "i64".to_string(),
                }],
            },
            location: Location {
                uri: Url::parse("file:///test.rs").unwrap(),
                range: Range {
                    start: Position {
                        line: 10,
                        character: 0,
                    },
                    end: Position {
                        line: 15,
                        character: 0,
                    },
                },
            },
        };

        assert_eq!(route.path, "/users/{id}");
        assert_eq!(route.methods.len(), 1);
        assert_eq!(route.methods[0], HttpMethod::Get);
        assert_eq!(route.handler.function_name, "get_user");
        assert_eq!(route.handler.parameters.len(), 1);
        assert_eq!(route.handler.parameters[0].name, "id");
        assert_eq!(route.handler.parameters[0].type_name, "i64");
    }

    #[test]
    fn test_handler_info_creation() {
        let handler = HandlerInfo {
            function_name: "create_user".to_string(),
            parameters: vec![
                Parameter {
                    name: "body".to_string(),
                    type_name: "Json<CreateUserRequest>".to_string(),
                },
                Parameter {
                    name: "db".to_string(),
                    type_name: "Component<ConnectPool>".to_string(),
                },
            ],
        };

        assert_eq!(handler.function_name, "create_user");
        assert_eq!(handler.parameters.len(), 2);
        assert_eq!(handler.parameters[0].name, "body");
        assert_eq!(handler.parameters[1].name, "db");
    }

    #[test]
    fn test_parameter_creation() {
        let param = Parameter {
            name: "user_id".to_string(),
            type_name: "Path<i64>".to_string(),
        };

        assert_eq!(param.name, "user_id");
        assert_eq!(param.type_name, "Path<i64>");
    }

    #[test]
    fn test_route_info_with_multiple_methods() {
        let route = RouteInfo {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Get, HttpMethod::Post],
            handler: HandlerInfo {
                function_name: "handle_users".to_string(),
                parameters: vec![],
            },
            location: Location {
                uri: Url::parse("file:///test.rs").unwrap(),
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 0,
                    },
                },
            },
        };

        assert_eq!(route.methods.len(), 2);
        assert!(route.methods.contains(&HttpMethod::Get));
        assert!(route.methods.contains(&HttpMethod::Post));
    }

    #[test]
    fn test_route_info_clone() {
        let route = RouteInfo {
            path: "/test".to_string(),
            methods: vec![HttpMethod::Get],
            handler: HandlerInfo {
                function_name: "test_handler".to_string(),
                parameters: vec![],
            },
            location: Location {
                uri: Url::parse("file:///test.rs").unwrap(),
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 0,
                    },
                },
            },
        };

        let cloned = route.clone();
        assert_eq!(route.path, cloned.path);
        assert_eq!(route.handler.function_name, cloned.handler.function_name);
    }

    #[test]
    fn test_route_index_with_routes() {
        let mut index = RouteIndex::new();

        // 添加路由
        let route1 = RouteInfo {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Get],
            handler: HandlerInfo {
                function_name: "list_users".to_string(),
                parameters: vec![],
            },
            location: Location {
                uri: Url::parse("file:///test.rs").unwrap(),
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 0,
                    },
                },
            },
        };

        let route2 = RouteInfo {
            path: "/users/{id}".to_string(),
            methods: vec![HttpMethod::Get],
            handler: HandlerInfo {
                function_name: "get_user".to_string(),
                parameters: vec![],
            },
            location: Location {
                uri: Url::parse("file:///test.rs").unwrap(),
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 0,
                    },
                },
            },
        };

        index.routes.push(route1);
        index.routes.push(route2);

        // 构建路径映射
        index.path_map.insert("/users".to_string(), vec![0]);
        index.path_map.insert("/users/{id}".to_string(), vec![1]);

        assert_eq!(index.routes.len(), 2);
        assert_eq!(index.path_map.len(), 2);
        assert_eq!(index.path_map.get("/users"), Some(&vec![0]));
        assert_eq!(index.path_map.get("/users/{id}"), Some(&vec![1]));
    }

    #[test]
    fn test_build_index_empty_documents() {
        let mut navigator = RouteNavigator::new();
        let documents = vec![];

        navigator.build_index(&documents);

        assert_eq!(navigator.index.routes.len(), 0);
        assert_eq!(navigator.index.path_map.len(), 0);
    }

    #[test]
    fn test_build_index_single_route() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route_macro = RouteMacro {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "list_users".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route_macro)],
        };

        navigator.build_index(&[doc]);

        assert_eq!(navigator.index.routes.len(), 1);
        assert_eq!(navigator.index.routes[0].path, "/users");
        assert_eq!(navigator.index.routes[0].methods.len(), 1);
        assert_eq!(navigator.index.routes[0].methods[0], HttpMethod::Get);
        assert_eq!(
            navigator.index.routes[0].handler.function_name,
            "list_users"
        );
        assert_eq!(navigator.index.path_map.len(), 1);
        assert_eq!(navigator.index.path_map.get("/users"), Some(&vec![0]));
    }

    #[test]
    fn test_build_index_with_path_parameters() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route_macro = RouteMacro {
            path: "/users/{id}".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "get_user".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route_macro)],
        };

        navigator.build_index(&[doc]);

        assert_eq!(navigator.index.routes.len(), 1);
        assert_eq!(navigator.index.routes[0].path, "/users/{id}");
        assert_eq!(navigator.index.routes[0].handler.parameters.len(), 1);
        assert_eq!(navigator.index.routes[0].handler.parameters[0].name, "id");
    }

    #[test]
    fn test_build_index_multiple_path_parameters() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route_macro = RouteMacro {
            path: "/users/{user_id}/posts/{post_id}".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "get_user_post".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route_macro)],
        };

        navigator.build_index(&[doc]);

        assert_eq!(navigator.index.routes.len(), 1);
        assert_eq!(navigator.index.routes[0].handler.parameters.len(), 2);
        assert_eq!(
            navigator.index.routes[0].handler.parameters[0].name,
            "user_id"
        );
        assert_eq!(
            navigator.index.routes[0].handler.parameters[1].name,
            "post_id"
        );
    }

    #[test]
    fn test_build_index_multi_method_route() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        // 路由宏包含多个 HTTP 方法
        let route_macro = RouteMacro {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Get, HttpMethod::Post],
            middlewares: vec![],
            handler_name: "handle_users".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route_macro)],
        };

        navigator.build_index(&[doc]);

        // 应该为每个方法创建独立的路由条目
        assert_eq!(navigator.index.routes.len(), 2);
        assert_eq!(navigator.index.routes[0].methods.len(), 1);
        assert_eq!(navigator.index.routes[0].methods[0], HttpMethod::Get);
        assert_eq!(navigator.index.routes[1].methods.len(), 1);
        assert_eq!(navigator.index.routes[1].methods[0], HttpMethod::Post);

        // 两个路由应该映射到同一个路径
        assert_eq!(navigator.index.path_map.get("/users"), Some(&vec![0, 1]));
    }

    #[test]
    fn test_build_index_multiple_routes() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route1 = RouteMacro {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "list_users".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let route2 = RouteMacro {
            path: "/users/{id}".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "get_user".to_string(),
            range: Range {
                start: Position {
                    line: 20,
                    character: 0,
                },
                end: Position {
                    line: 25,
                    character: 0,
                },
            },
        };

        let route3 = RouteMacro {
            path: "/posts".to_string(),
            methods: vec![HttpMethod::Get, HttpMethod::Post],
            middlewares: vec![],
            handler_name: "handle_posts".to_string(),
            range: Range {
                start: Position {
                    line: 30,
                    character: 0,
                },
                end: Position {
                    line: 35,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![
                SpringMacro::Route(route1),
                SpringMacro::Route(route2),
                SpringMacro::Route(route3),
            ],
        };

        navigator.build_index(&[doc]);

        // 应该有 4 个路由条目（route3 有 2 个方法）
        assert_eq!(navigator.index.routes.len(), 4);
        assert_eq!(navigator.index.path_map.len(), 3);
        assert_eq!(navigator.index.path_map.get("/users"), Some(&vec![0]));
        assert_eq!(navigator.index.path_map.get("/users/{id}"), Some(&vec![1]));
        assert_eq!(navigator.index.path_map.get("/posts"), Some(&vec![2, 3]));
    }

    #[test]
    fn test_build_index_multiple_documents() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route1 = RouteMacro {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "list_users".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc1 = RustDocument {
            uri: Url::parse("file:///users.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route1)],
        };

        let route2 = RouteMacro {
            path: "/posts".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "list_posts".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc2 = RustDocument {
            uri: Url::parse("file:///posts.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route2)],
        };

        navigator.build_index(&[doc1, doc2]);

        assert_eq!(navigator.index.routes.len(), 2);
        assert_eq!(navigator.index.path_map.len(), 2);
    }

    #[test]
    fn test_build_index_rebuild_clears_old_index() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        // 第一次构建
        let route1 = RouteMacro {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "list_users".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc1 = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route1)],
        };

        navigator.build_index(&[doc1]);
        assert_eq!(navigator.index.routes.len(), 1);

        // 第二次构建（应该清空旧索引）
        let route2 = RouteMacro {
            path: "/posts".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "list_posts".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc2 = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route2)],
        };

        navigator.build_index(&[doc2]);

        // 应该只有新的路由
        assert_eq!(navigator.index.routes.len(), 1);
        assert_eq!(navigator.index.routes[0].path, "/posts");
        assert_eq!(navigator.index.path_map.len(), 1);
        assert!(navigator.index.path_map.contains_key("/posts"));
        assert!(!navigator.index.path_map.contains_key("/users"));
    }

    #[test]
    fn test_parse_path_parameters_no_params() {
        let navigator = RouteNavigator::new();
        let params = navigator.parse_path_parameters("/users");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_parse_path_parameters_single_param() {
        let navigator = RouteNavigator::new();
        let params = navigator.parse_path_parameters("/users/{id}");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], "id");
    }

    #[test]
    fn test_parse_path_parameters_multiple_params() {
        let navigator = RouteNavigator::new();
        let params = navigator.parse_path_parameters("/users/{user_id}/posts/{post_id}");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], "user_id");
        assert_eq!(params[1], "post_id");
    }

    #[test]
    fn test_parse_path_parameters_empty_param() {
        let navigator = RouteNavigator::new();
        let params = navigator.parse_path_parameters("/users/{}");
        // 空参数名应该被忽略
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_parse_path_parameters_complex_path() {
        let navigator = RouteNavigator::new();
        let params = navigator
            .parse_path_parameters("/api/v1/users/{user_id}/posts/{post_id}/comments/{comment_id}");
        assert_eq!(params.len(), 3);
        assert_eq!(params[0], "user_id");
        assert_eq!(params[1], "post_id");
        assert_eq!(params[2], "comment_id");
    }

    // ============================================================================
    // 路由查找功能测试
    // ============================================================================

    #[test]
    fn test_get_all_routes_empty() {
        let navigator = RouteNavigator::new();
        let routes = navigator.get_all_routes();
        assert_eq!(routes.len(), 0);
    }

    #[test]
    fn test_get_all_routes() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route1 = RouteMacro {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "list_users".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let route2 = RouteMacro {
            path: "/posts".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "list_posts".to_string(),
            range: Range {
                start: Position {
                    line: 20,
                    character: 0,
                },
                end: Position {
                    line: 25,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route1), SpringMacro::Route(route2)],
        };

        navigator.build_index(&[doc]);

        let routes = navigator.get_all_routes();
        assert_eq!(routes.len(), 2);
        assert_eq!(routes[0].path, "/users");
        assert_eq!(routes[1].path, "/posts");
    }

    #[test]
    fn test_find_routes_empty_pattern() {
        let navigator = RouteNavigator::new();
        let routes = navigator.find_routes("");
        assert_eq!(routes.len(), 0);
    }

    #[test]
    fn test_find_routes_fuzzy_match() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let routes_data = vec![
            ("/users", "list_users"),
            ("/users/{id}", "get_user"),
            ("/posts", "list_posts"),
            ("/api/users", "api_list_users"),
        ];

        let macros: Vec<_> = routes_data
            .into_iter()
            .map(|(path, handler)| {
                SpringMacro::Route(RouteMacro {
                    path: path.to_string(),
                    methods: vec![HttpMethod::Get],
                    middlewares: vec![],
                    handler_name: handler.to_string(),
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 0,
                        },
                    },
                })
            })
            .collect();

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros,
        };

        navigator.build_index(&[doc]);

        // 模糊匹配 "users"
        let routes = navigator.find_routes("users");
        assert_eq!(routes.len(), 3); // /users, /users/{id}, /api/users

        // 模糊匹配 "posts"
        let routes = navigator.find_routes("posts");
        assert_eq!(routes.len(), 1); // /posts

        // 模糊匹配 "/api"
        let routes = navigator.find_routes("/api");
        assert_eq!(routes.len(), 1); // /api/users
    }

    #[test]
    fn test_find_routes_regex_match() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let routes_data = vec![
            ("/users", "list_users"),
            ("/users/{id}", "get_user"),
            ("/posts", "list_posts"),
            ("/api/v1/users", "api_v1_users"),
            ("/api/v2/users", "api_v2_users"),
        ];

        let macros: Vec<_> = routes_data
            .into_iter()
            .map(|(path, handler)| {
                SpringMacro::Route(RouteMacro {
                    path: path.to_string(),
                    methods: vec![HttpMethod::Get],
                    middlewares: vec![],
                    handler_name: handler.to_string(),
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 0,
                        },
                    },
                })
            })
            .collect();

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros,
        };

        navigator.build_index(&[doc]);

        // 正则表达式匹配以 /api 开头的路由
        let routes = navigator.find_routes("regex:^/api/.*");
        assert_eq!(routes.len(), 2); // /api/v1/users, /api/v2/users

        // 正则表达式匹配包含参数的路由
        let routes = navigator.find_routes("regex:.*\\{.*\\}.*");
        assert_eq!(routes.len(), 1); // /users/{id}

        // 正则表达式匹配以 /users 开头的路由
        let routes = navigator.find_routes("regex:^/users");
        assert_eq!(routes.len(), 2); // /users, /users/{id}
    }

    #[test]
    fn test_find_routes_regex_invalid() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route = RouteMacro {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "list_users".to_string(),
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route)],
        };

        navigator.build_index(&[doc]);

        // 无效的正则表达式应该返回空列表
        let routes = navigator.find_routes("regex:[invalid");
        assert_eq!(routes.len(), 0);
    }

    #[test]
    fn test_find_routes_no_match() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route = RouteMacro {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "list_users".to_string(),
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route)],
        };

        navigator.build_index(&[doc]);

        // 不匹配的模式应该返回空列表
        let routes = navigator.find_routes("posts");
        assert_eq!(routes.len(), 0);

        let routes = navigator.find_routes("regex:^/api/.*");
        assert_eq!(routes.len(), 0);
    }

    #[test]
    fn test_find_routes_by_handler_empty() {
        let navigator = RouteNavigator::new();
        let routes = navigator.find_routes_by_handler("get_user");
        assert_eq!(routes.len(), 0);
    }

    #[test]
    fn test_find_routes_by_handler_single() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route = RouteMacro {
            path: "/users/{id}".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "get_user".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route)],
        };

        navigator.build_index(&[doc]);

        let routes = navigator.find_routes_by_handler("get_user");
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].path, "/users/{id}");
        assert_eq!(routes[0].handler.function_name, "get_user");
    }

    #[test]
    fn test_find_routes_by_handler_multiple() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        // 同一个处理器处理多个路由
        let route1 = RouteMacro {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Get, HttpMethod::Post],
            middlewares: vec![],
            handler_name: "handle_users".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route1)],
        };

        navigator.build_index(&[doc]);

        let routes = navigator.find_routes_by_handler("handle_users");
        assert_eq!(routes.len(), 2); // GET 和 POST 各一个

        for route in routes {
            assert_eq!(route.path, "/users");
            assert_eq!(route.handler.function_name, "handle_users");
        }
    }

    #[test]
    fn test_find_routes_by_handler_not_found() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route = RouteMacro {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "list_users".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route)],
        };

        navigator.build_index(&[doc]);

        let routes = navigator.find_routes_by_handler("get_user");
        assert_eq!(routes.len(), 0);
    }

    #[test]
    fn test_route_location_for_jump() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route = RouteMacro {
            path: "/users/{id}".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "get_user".to_string(),
            range: Range {
                start: Position {
                    line: 42,
                    character: 5,
                },
                end: Position {
                    line: 50,
                    character: 10,
                },
            },
        };

        let uri = Url::parse("file:///src/handlers/users.rs").unwrap();

        let doc = RustDocument {
            uri: uri.clone(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route)],
        };

        navigator.build_index(&[doc]);

        let routes = navigator.find_routes("users");
        assert_eq!(routes.len(), 1);

        // 验证位置信息可用于跳转
        let location = &routes[0].location;
        assert_eq!(location.uri, uri);
        assert_eq!(location.range.start.line, 42);
        assert_eq!(location.range.start.character, 5);
        assert_eq!(location.range.end.line, 50);
        assert_eq!(location.range.end.character, 10);
    }

    // ============================================================================
    // 路由验证功能测试
    // ============================================================================

    #[test]
    fn test_validate_routes_empty() {
        let navigator = RouteNavigator::new();
        let diagnostics = navigator.validate_routes();
        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn test_validate_path_characters_valid() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route = RouteMacro {
            path: "/api/v1/users/{id}/posts".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "get_user_posts".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route)],
        };

        navigator.build_index(&[doc]);

        let diagnostics = navigator.validate_routes();
        // 不应该有路径字符错误
        assert!(!diagnostics.iter().any(|d| d
            .code
            .as_ref()
            .and_then(|c| match c {
                lsp_types::NumberOrString::String(s) => Some(s.as_str()),
                _ => None,
            })
            .map(|s| s == "invalid-path-char")
            .unwrap_or(false)));
    }

    #[test]
    fn test_validate_path_characters_invalid() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route = RouteMacro {
            path: "/users/<id>".to_string(), // < 和 > 是无效字符
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "get_user".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route)],
        };

        navigator.build_index(&[doc]);

        let diagnostics = navigator.validate_routes();
        // 应该有路径字符错误
        assert!(diagnostics.iter().any(|d| d
            .code
            .as_ref()
            .and_then(|c| match c {
                lsp_types::NumberOrString::String(s) => Some(s.as_str()),
                _ => None,
            })
            .map(|s| s == "invalid-path-char")
            .unwrap_or(false)));
    }

    #[test]
    fn test_validate_path_parameter_syntax_valid() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route = RouteMacro {
            path: "/users/{id}/posts/{post_id}".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "get_user_post".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route)],
        };

        navigator.build_index(&[doc]);

        let diagnostics = navigator.validate_routes();
        // 不应该有参数语法错误
        assert!(!diagnostics.iter().any(|d| {
            if let Some(lsp_types::NumberOrString::String(code)) = &d.code {
                code.contains("path-param") || code.contains("brace")
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_validate_path_parameter_syntax_empty_param() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route = RouteMacro {
            path: "/users/{}".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "get_user".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route)],
        };

        navigator.build_index(&[doc]);

        let diagnostics = navigator.validate_routes();
        // 应该有空参数名错误
        assert!(diagnostics.iter().any(|d| d
            .code
            .as_ref()
            .and_then(|c| match c {
                lsp_types::NumberOrString::String(s) => Some(s.as_str()),
                _ => None,
            })
            .map(|s| s == "empty-path-param")
            .unwrap_or(false)));
    }

    #[test]
    fn test_validate_path_parameter_syntax_unclosed() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route = RouteMacro {
            path: "/users/{id".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "get_user".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route)],
        };

        navigator.build_index(&[doc]);

        let diagnostics = navigator.validate_routes();
        // 应该有未闭合括号错误
        assert!(diagnostics.iter().any(|d| d
            .code
            .as_ref()
            .and_then(|c| match c {
                lsp_types::NumberOrString::String(s) => Some(s.as_str()),
                _ => None,
            })
            .map(|s| s == "unclosed-path-param")
            .unwrap_or(false)));
    }

    #[test]
    fn test_validate_path_parameter_syntax_unmatched_closing() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route = RouteMacro {
            path: "/users/id}".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "get_user".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route)],
        };

        navigator.build_index(&[doc]);

        let diagnostics = navigator.validate_routes();
        // 应该有未匹配的闭括号错误
        assert!(diagnostics.iter().any(|d| d
            .code
            .as_ref()
            .and_then(|c| match c {
                lsp_types::NumberOrString::String(s) => Some(s.as_str()),
                _ => None,
            })
            .map(|s| s == "unmatched-closing-brace")
            .unwrap_or(false)));
    }

    #[test]
    fn test_validate_path_parameter_syntax_nested() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route = RouteMacro {
            path: "/users/{{id}}".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "get_user".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route)],
        };

        navigator.build_index(&[doc]);

        let diagnostics = navigator.validate_routes();
        // 应该有嵌套括号错误
        assert!(diagnostics.iter().any(|d| d
            .code
            .as_ref()
            .and_then(|c| match c {
                lsp_types::NumberOrString::String(s) => Some(s.as_str()),
                _ => None,
            })
            .map(|s| s == "nested-path-param")
            .unwrap_or(false)));
    }

    #[test]
    fn test_validate_restful_style_valid() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route = RouteMacro {
            path: "/api/v1/users/{id}/posts".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "get_user_posts".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route)],
        };

        navigator.build_index(&[doc]);

        let diagnostics = navigator.validate_routes();
        // 不应该有 RESTful 风格警告
        assert!(!diagnostics.iter().any(|d| {
            if let Some(lsp_types::NumberOrString::String(code)) = &d.code {
                code.starts_with("restful-style")
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_validate_restful_style_verb() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route = RouteMacro {
            path: "/getUsers".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "get_users".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route)],
        };

        navigator.build_index(&[doc]);

        let diagnostics = navigator.validate_routes();
        // 应该有动词使用警告
        assert!(diagnostics.iter().any(|d| d
            .code
            .as_ref()
            .and_then(|c| match c {
                lsp_types::NumberOrString::String(s) => Some(s.as_str()),
                _ => None,
            })
            .map(|s| s == "restful-style-verb")
            .unwrap_or(false)));
    }

    #[test]
    fn test_validate_restful_style_case() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route = RouteMacro {
            path: "/userProfiles".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "get_user_profiles".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route)],
        };

        navigator.build_index(&[doc]);

        let diagnostics = navigator.validate_routes();
        // 应该有大写字母使用警告
        assert!(diagnostics.iter().any(|d| d
            .code
            .as_ref()
            .and_then(|c| match c {
                lsp_types::NumberOrString::String(s) => Some(s.as_str()),
                _ => None,
            })
            .map(|s| s == "restful-style-case")
            .unwrap_or(false)));
    }

    #[test]
    fn test_detect_conflicts_empty() {
        let navigator = RouteNavigator::new();
        let conflicts = navigator.detect_conflicts();
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_detect_conflicts_no_conflict() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route1 = RouteMacro {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "list_users".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let route2 = RouteMacro {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Post],
            middlewares: vec![],
            handler_name: "create_user".to_string(),
            range: Range {
                start: Position {
                    line: 20,
                    character: 0,
                },
                end: Position {
                    line: 25,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route1), SpringMacro::Route(route2)],
        };

        navigator.build_index(&[doc]);

        let conflicts = navigator.detect_conflicts();
        // 不同的 HTTP 方法，不应该有冲突
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_detect_conflicts_same_path_and_method() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        let route1 = RouteMacro {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "list_users".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let route2 = RouteMacro {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "get_users".to_string(),
            range: Range {
                start: Position {
                    line: 20,
                    character: 0,
                },
                end: Position {
                    line: 25,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![SpringMacro::Route(route1), SpringMacro::Route(route2)],
        };

        navigator.build_index(&[doc]);

        let conflicts = navigator.detect_conflicts();
        // 相同的路径和方法，应该有冲突
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].path, "/users");
        assert_eq!(conflicts[0].method, HttpMethod::Get);
    }

    #[test]
    fn test_detect_conflicts_multiple() {
        use crate::macro_analyzer::{RouteMacro, RustDocument, SpringMacro};

        let mut navigator = RouteNavigator::new();

        // 三个路由，都是 GET /users
        let route1 = RouteMacro {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "handler1".to_string(),
            range: Range {
                start: Position {
                    line: 10,
                    character: 0,
                },
                end: Position {
                    line: 15,
                    character: 0,
                },
            },
        };

        let route2 = RouteMacro {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "handler2".to_string(),
            range: Range {
                start: Position {
                    line: 20,
                    character: 0,
                },
                end: Position {
                    line: 25,
                    character: 0,
                },
            },
        };

        let route3 = RouteMacro {
            path: "/users".to_string(),
            methods: vec![HttpMethod::Get],
            middlewares: vec![],
            handler_name: "handler3".to_string(),
            range: Range {
                start: Position {
                    line: 30,
                    character: 0,
                },
                end: Position {
                    line: 35,
                    character: 0,
                },
            },
        };

        let doc = RustDocument {
            uri: Url::parse("file:///test.rs").unwrap(),
            content: String::new(),
            macros: vec![
                SpringMacro::Route(route1),
                SpringMacro::Route(route2),
                SpringMacro::Route(route3),
            ],
        };

        navigator.build_index(&[doc]);

        let conflicts = navigator.detect_conflicts();
        // 应该有 3 个冲突：(0,1), (0,2), (1,2)
        assert_eq!(conflicts.len(), 3);
    }

    #[test]
    fn test_route_conflict_creation() {
        let conflict = RouteConflict {
            index1: 0,
            index2: 1,
            path: "/users".to_string(),
            method: HttpMethod::Get,
            location1: Location {
                uri: Url::parse("file:///test.rs").unwrap(),
                range: Range {
                    start: Position {
                        line: 10,
                        character: 0,
                    },
                    end: Position {
                        line: 15,
                        character: 0,
                    },
                },
            },
            location2: Location {
                uri: Url::parse("file:///test.rs").unwrap(),
                range: Range {
                    start: Position {
                        line: 20,
                        character: 0,
                    },
                    end: Position {
                        line: 25,
                        character: 0,
                    },
                },
            },
        };

        assert_eq!(conflict.index1, 0);
        assert_eq!(conflict.index2, 1);
        assert_eq!(conflict.path, "/users");
        assert_eq!(conflict.method, HttpMethod::Get);
    }
}
