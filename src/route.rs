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
                parameters: vec![
                    Parameter {
                        name: "id".to_string(),
                        type_name: "i64".to_string(),
                    },
                ],
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
}
