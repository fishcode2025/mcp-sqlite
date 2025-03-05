/*!
 * 这个示例展示了如何使用mcp-sqlite库作为一个库来创建自定义的SQLite MCP服务器。
 */

use mcp_server_fishcode2025::{router::RouterService, ByteTransport, Server};
use mcp_sqlite::server::SQLiteRouter;
use tokio::io::{stdin, stdout};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("设置全局默认日志订阅者失败");

    info!("启动自定义SQLite MCP服务器");

    // 创建SQLite路由器，使用内存数据库
    let router = SQLiteRouter::new(":memory:")?;

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
