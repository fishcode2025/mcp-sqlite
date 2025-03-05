use mcp_client::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端
    let mut client = Client::connect("127.0.0.1:8080").await?;
    println!("已连接到SQLite MCP服务器");

    // 创建表
    println!("创建users表...");
    client.call("execute", json!({
        "statement": "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)"
    })).await?;

    // 插入数据
    println!("插入数据...");
    let result = client
        .call(
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
        .call(
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
        .call(
            "query",
            json!({
                "query": "SELECT * FROM users"
            }),
        )
        .await?;
    println!("查询结果:");

    // 解析并打印结果
    if let Some(columns) = result.get("columns").and_then(|v| v.as_array()) {
        if let Some(rows) = result.get("rows").and_then(|v| v.as_array()) {
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

    // 使用参数查询
    println!("\n按名称查询用户...");
    let result = client
        .call(
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
        .call(
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
        .call(
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
