//! 自定义 LSP 请求处理器
//!
//! 处理 spring-rs 特定的自定义请求

use lsp_server::{Request, Response};

/// 处理自定义请求
pub fn handle_custom_request(req: Request) -> Option<Response> {
    match req.method.as_str() {
        "spring/components" => handle_components_request(req),
        "spring/routes" => handle_routes_request(req),
        "spring/jobs" => handle_jobs_request(req),
        "spring/plugins" => handle_plugins_request(req),
        _ => None,
    }
}

fn handle_components_request(_req: Request) -> Option<Response> {
    // TODO: 实现组件请求处理
    tracing::debug!("Handling spring/components request");
    None
}

fn handle_routes_request(_req: Request) -> Option<Response> {
    // TODO: 实现路由请求处理
    tracing::debug!("Handling spring/routes request");
    None
}

fn handle_jobs_request(_req: Request) -> Option<Response> {
    // TODO: 实现任务请求处理
    tracing::debug!("Handling spring/jobs request");
    None
}

fn handle_plugins_request(_req: Request) -> Option<Response> {
    // TODO: 实现插件请求处理
    tracing::debug!("Handling spring/plugins request");
    None
}
