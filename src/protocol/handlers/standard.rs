//! 标准 LSP 请求处理器
//!
//! 处理标准的 LSP 协议请求

use lsp_server::{Request, Response};

/// 处理标准 LSP 请求
pub fn handle_standard_request(req: Request) -> Option<Response> {
    match req.method.as_str() {
        "textDocument/completion" => handle_completion(req),
        "textDocument/hover" => handle_hover(req),
        "textDocument/definition" => handle_definition(req),
        "textDocument/references" => handle_references(req),
        "textDocument/rename" => handle_rename(req),
        _ => None,
    }
}

fn handle_completion(_req: Request) -> Option<Response> {
    // TODO: 实现补全处理
    tracing::debug!("Handling textDocument/completion request");
    None
}

fn handle_hover(_req: Request) -> Option<Response> {
    // TODO: 实现悬停提示处理
    tracing::debug!("Handling textDocument/hover request");
    None
}

fn handle_definition(_req: Request) -> Option<Response> {
    // TODO: 实现跳转到定义处理
    tracing::debug!("Handling textDocument/definition request");
    None
}

fn handle_references(_req: Request) -> Option<Response> {
    // TODO: 实现查找引用处理
    tracing::debug!("Handling textDocument/references request");
    None
}

fn handle_rename(_req: Request) -> Option<Response> {
    // TODO: 实现重命名处理
    tracing::debug!("Handling textDocument/rename request");
    None
}
