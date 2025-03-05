/*!
 * # SQLite MCP服务器
 *
 * 这是一个使用Rust实现的SQLite MCP（Model Context Protocol）服务器，
 * 提供通过MCP协议访问SQLite数据库的能力。
 *
 * ## 功能
 *
 * 服务器提供以下MCP方法：
 *
 * - `query`: 执行SQL查询并返回结果
 * - `execute`: 执行SQL语句
 * - `executemany`: 使用不同参数多次执行SQL语句
 * - `executescript`: 执行SQL脚本
 *
 * ## 使用方法
 *
 * ```bash
 * # 使用内存数据库
 * ./mcp-sqlite
 *
 * # 使用指定的SQLite数据库文件
 * ./mcp-sqlite --db path/to/database.db
 * ```
 *
 * ## 命令行选项
 *
 * - `--db`: SQLite数据库文件路径（默认为内存数据库`:memory:`）
 * - `--log-level`: 日志级别（默认为`info`）
 */

mod server;

use clap::Parser;
use mcp_server_fishcode2025::{router::RouterService, ByteTransport, Server};
use server::SQLiteRouter;
use tokio::io::{stdin, stdout};
use tracing::{error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// SQLite MCP服务器命令行参数
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// SQLite数据库文件路径，使用":memory:"表示内存数据库
    #[arg(short, long, default_value = ":memory:")]
    db: String,

    /// 日志级别，可选值：trace, debug, info, warn, error
    #[arg(long, default_value = "info")]
    log_level: String,
}

/// 程序入口点
///
/// 解析命令行参数，设置日志，创建SQLite路由器，并启动MCP服务器
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
