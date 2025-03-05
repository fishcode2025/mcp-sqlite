# SQLite MCP服务器

这是一个使用Rust实现的SQLite MCP服务器，提供通过Model Context Protocol访问SQLite数据库的能力。

## 功能

服务器提供以下MCP方法：

### `query`

执行SQL查询并返回结果。

#### 参数

- `query`：要执行的SQL查询。
- `params`：（可选）绑定到查询的参数。

#### 返回

- `columns`：列名。
- `rows`：查询返回的行。

### `execute`

执行SQL语句。

#### 参数

- `statement`：要执行的SQL语句。
- `params`：（可选）绑定到语句的参数。

#### 返回

- `rowcount`：受影响的行数。
- `lastrowid`：最后插入行的ID（如适用）。

### `executemany`

使用不同参数多次执行SQL语句。

#### 参数

- `statement`：要执行的SQL语句。
- `params_list`：绑定到语句的参数列表。

#### 返回

- `rowcount`：受影响的行数。

### `executescript`

执行SQL脚本。

#### 参数

- `script`：要执行的SQL脚本。

#### 返回

- `rowcount`：受影响的行数。

## 安装

### 从源代码构建

```bash
git clone https://github.com/yourusername/mcp-sqlite.git
cd mcp-sqlite
cargo build --release
```

## 使用

### 启动服务器

```bash
./mcp-sqlite --db path/to/database.db --host 127.0.0.1 --port 8080
```

### 命令行选项

- `--db`：SQLite数据库文件路径（默认为内存数据库`:memory:`）
- `--host`：服务器绑定地址（默认为`127.0.0.1`）
- `--port`：服务器绑定端口（默认为`8080`）
- `--log-level`：日志级别（默认为`info`）

### 客户端示例

```rust
use mcp_client::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端
    let mut client = Client::connect("127.0.0.1:8080").await?;

    // 创建表
    client.call("execute", json!({
        "statement": "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)"
    })).await?;

    // 插入数据
    client.call("execute", json!({
        "statement": "INSERT INTO users (name, email) VALUES (?, ?)",
        "params": ["张三", "zhangsan@example.com"]
    })).await?;

    // 查询数据
    let result = client.call("query", json!({
        "query": "SELECT * FROM users"
    })).await?;

    println!("查询结果: {:?}", result);

    Ok(())
}
```

## 许可证

MIT 