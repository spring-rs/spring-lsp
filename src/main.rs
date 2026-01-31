use anyhow::Result;
use spring_lsp::server::LspServer;

fn main() -> Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting spring-lsp language server");

    // 启动 LSP 服务器
    let mut server = LspServer::start()?;
    server.run()?;

    tracing::info!("spring-lsp language server stopped");
    Ok(())
}
