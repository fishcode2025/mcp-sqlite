mod server;

use clap::Parser;
use mcp_server::{router::RouterService, ByteTransport, Server};
use server::SQLiteRouter;
use tokio::io::{stdin, stdout};
use tracing::{error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// SQLite MCP服务器
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// SQLite数据库文件路径
    #[arg(short, long, default_value = ":memory:")]
    db: String,

    /// 服务器绑定地址
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// 服务器绑定端口
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// 日志级别
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 解析命令行参数
    let args = Args::parse();

    // 设置日志
    let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "mcp-sqlite.log");

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(format!("mcp_sqlite={}", args.log_level))),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(file_appender))
        .init();

    info!("Starting SQLite MCP Server");
    info!("Database: {}", args.db);

    // 创建服务器
    let router = match SQLiteRouter::new(&args.db) {
        Ok(router) => router,
        Err(e) => {
            error!("Failed to create SQLite router: {}", e);
            return Err(anyhow::anyhow!("Failed to create SQLite router: {}", e));
        }
    };

    // 创建路由服务
    let router_service = RouterService(router);

    // 创建服务器
    let server = Server::new(router_service);

    // 创建传输层
    let transport = ByteTransport::new(stdin(), stdout());

    info!("Server started successfully");

    // 运行服务器
    Ok(server.run(transport).await?)
}
