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
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::try_new(&args.log_level).unwrap_or_else(|_| EnvFilter::new("info"))
        }))
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .init();

    info!("启动SQLite MCP服务器");
    info!("数据库路径: {}", args.db);

    // 创建SQLite路由器
    let router = match SQLiteRouter::new(&args.db) {
        Ok(router) => router,
        Err(e) => {
            error!("创建SQLite路由器失败: {}", e);
            return Err(anyhow::anyhow!("创建SQLite路由器失败: {}", e));
        }
    };

    // 创建路由服务
    let router_service = RouterService(router);

    // 创建服务器
    let server = Server::new(router_service);

    // 使用标准输入输出作为传输层
    let transport = ByteTransport::new(stdin(), stdout());

    // 运行服务器
    info!("服务器已启动，使用stdio传输");
    Ok(server.run(transport).await?)
}
