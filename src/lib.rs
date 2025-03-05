/*!
 * # SQLite MCP服务器库
 *
 * 这是一个使用Rust实现的SQLite MCP（Model Context Protocol）服务器库，
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
 * ### 作为库使用
 *
 * ```rust
 * use mcp_sqlite::server::SQLiteRouter;
 * use mcp_server_fishcode2025::{router::RouterService, ByteTransport, Server};
 * use tokio::io::{stdin, stdout};
 *
 * #[tokio::main]
 * async fn main() -> anyhow::Result<()> {
 *     // 创建SQLite路由器
 *     let router = SQLiteRouter::new(":memory:")?;
 *     
 *     // 创建路由服务
 *     let router_service = RouterService(router);
 *     
 *     // 创建服务器
 *     let server = Server::new(router_service);
 *     
 *     // 使用标准输入输出作为传输层
 *     let transport = ByteTransport::new(stdin(), stdout());
 *     
 *     // 运行服务器
 *     Ok(server.run(transport).await?)
 * }
 * ```
 *
 * ### 作为可执行文件使用
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

// 注释掉这一行，因为它需要nightly版本的Rust
// #![cfg_attr(docsrs, feature(doc_cfg))]

/// SQLite MCP服务器实现
pub mod server;

// 重新导出主要类型，方便用户使用
pub use server::SQLiteRouter;
