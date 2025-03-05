use mcp_client_fishcode2025::{
    client::{ClientCapabilities, ClientInfo, McpClient, McpClientTrait},
    transport::{StdioTransport, Transport},
    McpService,
};
use serde_json::json;
use std::{collections::HashMap, env, time::Duration};

fn print_usage() {
    println!("SQLite MCP 客户端示例");
    println!("用法:");
    println!("  cargo run --example client -- [服务器路径] [选项]");
    println!();
    println!("参数:");
    println!("  [服务器路径]        SQLite MCP 服务器可执行文件的路径 (默认: ./mcp-sqlite)");
    println!();
    println!("选项:");
    println!("  --db <路径>         SQLite 数据库文件路径 (默认: :memory:)");
    println!("  --help              显示此帮助信息");
    println!();
    println!("示例:");
    println!("  cargo run --example client -- ./target/release/mcp-sqlite");
    println!("  cargo run --example client -- ./target/release/mcp-sqlite --db ./test.db");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();

    // 检查是否请求帮助
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        print_usage();
        return Ok(());
    }

    // 检查是否提供了服务器可执行文件路径
    let server_path = if args.len() > 1 && !args[1].starts_with("-") {
        args[1].clone()
    } else {
        // 默认使用当前目录下的mcp-sqlite可执行文件
        "./mcp-sqlite".to_string()
    };

    // 解析命令行参数
    let mut db_path = ":memory:".to_string();
    let mut server_args = Vec::new();
    let mut i = 2;

    while i < args.len() {
        match args[i].as_str() {
            "--db" => {
                if i + 1 < args.len() {
                    db_path = args[i + 1].clone();
                    i += 2;
                } else {
                    println!("错误: --db 参数需要指定数据库路径");
                    print_usage();
                    return Ok(());
                }
            }
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            arg => {
                // 其他参数直接传递给服务器
                server_args.push(arg.to_string());
                i += 1;
            }
        }
    }

    // 添加数据库路径参数
    server_args.push("--db".to_string());
    server_args.push(db_path.clone());

    println!("使用服务器: {}", server_path);
    println!("使用数据库: {}", db_path);
    println!("服务器参数: {:?}", server_args);

    // 创建客户端
    let transport = StdioTransport::new(server_path, server_args, HashMap::new());
    let handle = transport.start().await?;
    let service = McpService::with_timeout(handle, Duration::from_secs(30));
    let mut client = McpClient::new(service);

    // 初始化客户端
    let info = ClientInfo {
        name: "mcp-sqlite-client".to_string(),
        version: "1.0.0".to_string(),
    };
    let capabilities = ClientCapabilities::default();
    client.initialize(info, capabilities).await?;

    println!("已连接到SQLite MCP服务器");

    // 创建表
    println!("创建users表...");
    let result = client.call_tool("execute", json!({
        "statement": "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)"
    })).await?;

    // 插入数据
    println!("插入数据...");
    let result = client
        .call_tool(
            "execute",
            json!({
                "statement": "INSERT INTO users (name, email) VALUES (?, ?)",
                "params": ["张三", "zhangsan@example.com"]
            }),
        )
        .await?;
    println!("插入结果: {:?}", result);

    // 批量插入数据
    println!("批量插入数据...");
    let result = client
        .call_tool(
            "executemany",
            json!({
                "statement": "INSERT INTO users (name, email) VALUES (?, ?)",
                "params_list": [
                    ["李四", "lisi@example.com"],
                    ["王五", "wangwu@example.com"],
                    ["赵六", "zhaoliu@example.com"]
                ]
            }),
        )
        .await?;
    println!("批量插入结果: {:?}", result);

    // 查询数据
    println!("查询所有用户...");
    let result = client
        .call_tool(
            "query",
            json!({
                "query": "SELECT * FROM users"
            }),
        )
        .await?;
    println!("查询结果:");

    // 解析并打印结果
    if let Some(content) = result.content.first() {
        match content {
            mcp_core_fishcode2025::content::Content::Text(text_content) => {
                if let Ok(json_value) =
                    serde_json::from_str::<serde_json::Value>(&text_content.text)
                {
                    if let (Some(columns), Some(rows)) = (
                        json_value.get("columns").and_then(|v| v.as_array()),
                        json_value.get("rows").and_then(|v| v.as_array()),
                    ) {
                        // 打印表头
                        let header: Vec<String> = columns
                            .iter()
                            .map(|c| c.as_str().unwrap_or("").to_string())
                            .collect();
                        println!("{}", header.join(" | "));

                        // 打印分隔线
                        println!("{}", "-".repeat(header.join(" | ").len()));

                        // 打印数据行
                        for row in rows {
                            if let Some(row_array) = row.as_array() {
                                let row_str: Vec<String> = row_array
                                    .iter()
                                    .map(|v| match v {
                                        serde_json::Value::Null => "NULL".to_string(),
                                        _ => v.to_string().replace('"', ""),
                                    })
                                    .collect();
                                println!("{}", row_str.join(" | "));
                            }
                        }
                    }
                }
            }
            _ => println!("无法解析结果内容"),
        }
    }

    // 使用参数查询
    println!("\n按名称查询用户...");
    let result = client
        .call_tool(
            "query",
            json!({
                "query": "SELECT * FROM users WHERE name = ?",
                "params": ["张三"]
            }),
        )
        .await?;
    println!("查询结果: {:?}", result);

    // 执行脚本
    println!("\n执行SQL脚本...");
    let result = client
        .call_tool(
            "executescript",
            json!({
                "script": "
            CREATE TABLE IF NOT EXISTS products (id INTEGER PRIMARY KEY, name TEXT, price REAL);
            INSERT INTO products (name, price) VALUES ('产品A', 99.99);
            INSERT INTO products (name, price) VALUES ('产品B', 199.99);
        "
            }),
        )
        .await?;
    println!("脚本执行结果: {:?}", result);

    // 查询新表
    println!("\n查询products表...");
    let result = client
        .call_tool(
            "query",
            json!({
                "query": "SELECT * FROM products"
            }),
        )
        .await?;
    println!("查询结果: {:?}", result);

    println!("\n示例完成");
    Ok(())
}
