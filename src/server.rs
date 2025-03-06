/*!
 * # SQLite MCP服务器实现
 *
 * 本模块实现了SQLite MCP服务器的核心功能，包括：
 *
 * - SQLite连接管理
 * - MCP方法实现（query, execute, executemany, executescript）
 * - 参数处理和结果格式化
 *
 * ## 主要组件
 *
 * - [`SQLiteRouter`][]: 实现MCP路由器接口，处理客户端请求
 *
 * ## 支持的MCP方法
 *
 * ### `query`
 *
 * 执行SQL查询并返回结果。
 *
 * #### 查询参数
 *
 * - `query`：要执行的SQL查询
 * - `params`：（可选）绑定到查询的参数
 *
 * #### 查询返回值
 *
 * - `columns`：列名
 * - `rows`：查询返回的行
 *
 * ### `execute`
 *
 * 执行SQL语句。
 *
 * #### 执行参数
 *
 * - `statement`：要执行的SQL语句
 * - `params`：（可选）绑定到语句的参数
 *
 * #### 执行返回值
 *
 * - `rowcount`：受影响的行数
 * - `lastrowid`：最后插入行的ID（如适用）
 *
 * ### `executemany`
 *
 * 使用不同参数多次执行SQL语句。
 *
 * #### 批量执行参数
 *
 * - `statement`：要执行的SQL语句
 * - `params_list`：绑定到语句的参数列表
 *
 * #### 批量执行返回值
 *
 * - `rowcount`：受影响的行数
 *
 * ### `executescript`
 *
 * 执行SQL脚本。
 *
 * #### 脚本参数
 *
 * - `script`：要执行的SQL脚本
 *
 * #### 脚本返回值
 *
 * - `rowcount`：受影响的行数
 */

use std::{future::Future, pin::Pin, sync::Arc};

use base64::{engine::general_purpose::STANDARD, Engine};
use mcp_core_fishcode2025::{
    handler::{PromptError, ResourceError, ToolError},
    prompt::Prompt,
    protocol::ServerCapabilities,
    Content, Resource, Tool,
};
use mcp_server_fishcode2025::router::CapabilitiesBuilder;
use rusqlite::{Connection, Row, ToSql};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tracing::debug;

/// SQLite MCP服务器路由器
///
/// 负责处理MCP客户端请求，执行SQL操作，并返回结果
pub struct SQLiteRouter {
    /// SQLite数据库连接
    conn: Arc<Mutex<Connection>>,
}

impl SQLiteRouter {
    /// 创建一个新的SQLite MCP服务器路由器
    ///
    /// # 参数
    ///
    /// * `db_path` - SQLite数据库文件路径，使用":memory:"表示内存数据库
    ///
    /// # 返回值
    ///
    /// 成功时返回`SQLiteRouter`实例，失败时返回SQLite错误
    ///
    /// # 示例
    ///
    /// ```
    /// use mcp_sqlite::server::SQLiteRouter;
    ///
    /// let router = SQLiteRouter::new(":memory:").expect("创建路由器失败");
    /// ```
    pub fn new(db_path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 执行SQL查询并返回结果
    ///
    /// # 参数
    ///
    /// * `params` - 包含查询参数的JSON对象，必须包含"query"字段，可选包含"params"字段
    ///
    /// # 返回值
    ///
    /// 成功时返回包含查询结果的JSON对象，失败时返回工具错误
    async fn query(&self, params: Value) -> Result<Value, ToolError> {
        // 获取查询参数
        let query = match params.get("query") {
            Some(Value::String(q)) => q,
            _ => {
                return Err(ToolError::InvalidParameters(
                    "Missing required parameter: query".into(),
                ))
            }
        };

        // 获取绑定参数
        let params_json = json!([]);
        let bind_params = params.get("params").unwrap_or(&params_json);
        let bind_params = match bind_params {
            Value::Array(arr) => arr,
            _ => {
                return Err(ToolError::InvalidParameters(
                    "params must be an array".into(),
                ))
            }
        };

        // 执行查询
        let conn = self.conn.lock().await;

        let mut stmt = match conn.prepare(query) {
            Ok(stmt) => stmt,
            Err(e) => {
                return Err(ToolError::ExecutionError(format!(
                    "Failed to prepare query: {}",
                    e
                )))
            }
        };

        // 将JSON参数转换为SQLite参数
        let sql_params: Vec<Box<dyn ToSql>> =
            bind_params.iter().map(|v| json_value_to_sql(v)).collect();

        let sql_params_refs: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();

        // 先获取列名，避免借用冲突
        let column_names: Vec<String> = {
            let names = stmt.column_names();
            names.iter().map(|s| s.to_string()).collect()
        };

        // 执行查询
        let mut rows = match stmt.query(sql_params_refs.as_slice()) {
            Ok(rows) => rows,
            Err(e) => {
                return Err(ToolError::ExecutionError(format!(
                    "Failed to execute query: {}",
                    e
                )))
            }
        };

        // 获取结果行
        let mut result_rows = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            let row_values = extract_row_values(row, &column_names);
            result_rows.push(row_values);
        }

        Ok(json!({
            "columns": column_names,
            "rows": result_rows,
        }))
    }

    /// 执行SQL语句
    async fn execute(&self, params: Value) -> Result<Value, ToolError> {
        // 获取语句参数
        let statement = match params.get("statement") {
            Some(Value::String(s)) => s,
            _ => {
                return Err(ToolError::InvalidParameters(
                    "Missing required parameter: statement".into(),
                ))
            }
        };

        // 获取绑定参数
        let params_json = json!([]);
        let bind_params = params.get("params").unwrap_or(&params_json);
        let bind_params = match bind_params {
            Value::Array(arr) => arr,
            _ => {
                return Err(ToolError::InvalidParameters(
                    "params must be an array".into(),
                ))
            }
        };

        // 执行语句
        let conn = self.conn.lock().await;

        // 将JSON参数转换为SQLite参数
        let sql_params: Vec<Box<dyn ToSql>> =
            bind_params.iter().map(|v| json_value_to_sql(v)).collect();

        let sql_params_refs: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();

        // 执行语句
        let result = conn.execute(statement, sql_params_refs.as_slice());

        match result {
            Ok(rows_affected) => {
                // 获取最后插入的行ID
                let last_insert_id = conn.last_insert_rowid();

                Ok(json!({
                    "rowcount": rows_affected,
                    "lastrowid": last_insert_id,
                }))
            }
            Err(e) => Err(ToolError::ExecutionError(format!(
                "Failed to execute statement: {}",
                e
            ))),
        }
    }

    /// 执行多个SQL语句
    async fn executemany(&self, params: Value) -> Result<Value, ToolError> {
        // 获取语句参数
        let statement = match params.get("statement") {
            Some(Value::String(s)) => s,
            _ => {
                return Err(ToolError::InvalidParameters(
                    "Missing required parameter: statement".into(),
                ))
            }
        };

        // 获取参数列表
        let params_list = match params.get("params_list") {
            Some(Value::Array(list)) => list,
            _ => {
                return Err(ToolError::InvalidParameters(
                    "Missing required parameter: params_list".into(),
                ))
            }
        };

        // 执行语句
        let conn = self.conn.lock().await;

        let mut stmt = match conn.prepare(statement) {
            Ok(stmt) => stmt,
            Err(e) => {
                return Err(ToolError::ExecutionError(format!(
                    "Failed to prepare statement: {}",
                    e
                )))
            }
        };

        let mut rows_affected = 0;

        for params_item in params_list {
            match params_item {
                Value::Array(params) => {
                    // 将JSON参数转换为SQLite参数
                    let sql_params: Vec<Box<dyn ToSql>> =
                        params.iter().map(|v| json_value_to_sql(v)).collect();

                    let sql_params_refs: Vec<&dyn ToSql> =
                        sql_params.iter().map(|p| p.as_ref()).collect();

                    match stmt.execute(sql_params_refs.as_slice()) {
                        Ok(count) => rows_affected += count,
                        Err(e) => {
                            return Err(ToolError::ExecutionError(format!(
                                "Failed to execute statement: {}",
                                e
                            )))
                        }
                    }
                }
                _ => {
                    return Err(ToolError::InvalidParameters(
                        "params_list must contain arrays".into(),
                    ))
                }
            }
        }

        Ok(json!({
            "rowcount": rows_affected,
        }))
    }

    /// 执行SQL脚本
    async fn executescript(&self, params: Value) -> Result<Value, ToolError> {
        // 获取脚本参数
        let script = match params.get("script") {
            Some(Value::String(s)) => s,
            _ => {
                return Err(ToolError::InvalidParameters(
                    "Missing required parameter: script".into(),
                ))
            }
        };

        // 执行脚本
        let conn = self.conn.lock().await;

        match conn.execute_batch(script) {
            Ok(_) => {
                // 由于execute_batch不返回受影响的行数，我们返回0
                Ok(json!({
                    "rowcount": 0,
                }))
            }
            Err(e) => Err(ToolError::ExecutionError(format!(
                "Failed to execute script: {}",
                e
            ))),
        }
    }
}

impl mcp_server_fishcode2025::Router for SQLiteRouter {
    fn name(&self) -> String {
        "sqlite".to_string()
    }

    fn instructions(&self) -> String {
        "SQLite数据库访问服务，提供执行SQL查询和语句的能力。".to_string()
    }

    fn capabilities(&self) -> ServerCapabilities {
        CapabilitiesBuilder::new().with_tools(true).build()
    }

    fn list_tools(&self) -> Vec<Tool> {
        vec![
            Tool::new(
                "query".to_string(),
                "执行SQL查询并返回结果".to_string(),
                json!({
                    "type": "object",
                    "required": ["query"],
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "要执行的SQL查询"
                        },
                        "params": {
                            "type": "array",
                            "description": "绑定到查询的参数"
                        }
                    }
                }),
            ),
            Tool::new(
                "execute".to_string(),
                "执行SQL语句".to_string(),
                json!({
                    "type": "object",
                    "required": ["statement"],
                    "properties": {
                        "statement": {
                            "type": "string",
                            "description": "要执行的SQL语句"
                        },
                        "params": {
                            "type": "array",
                            "description": "绑定到语句的参数"
                        }
                    }
                }),
            ),
            Tool::new(
                "executemany".to_string(),
                "使用不同参数多次执行SQL语句".to_string(),
                json!({
                    "type": "object",
                    "required": ["statement", "params_list"],
                    "properties": {
                        "statement": {
                            "type": "string",
                            "description": "要执行的SQL语句"
                        },
                        "params_list": {
                            "type": "array",
                            "description": "绑定到语句的参数列表"
                        }
                    }
                }),
            ),
            Tool::new(
                "executescript".to_string(),
                "执行SQL脚本".to_string(),
                json!({
                    "type": "object",
                    "required": ["script"],
                    "properties": {
                        "script": {
                            "type": "string",
                            "description": "要执行的SQL脚本"
                        }
                    }
                }),
            ),
        ]
    }

    fn call_tool(
        &self,
        tool_name: &str,
        arguments: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Content>, ToolError>> + Send + 'static>> {
        let self_clone = self.clone();
        let tool_name = tool_name.to_string(); // 获取所有权

        Box::pin(async move {
            debug!("Calling tool: {}", tool_name);

            let result = match tool_name.as_str() {
                "query" => self_clone.query(arguments).await?,
                "execute" => self_clone.execute(arguments).await?,
                "executemany" => self_clone.executemany(arguments).await?,
                "executescript" => self_clone.executescript(arguments).await?,
                _ => return Err(ToolError::NotFound(format!("Unknown tool: {}", tool_name))),
            };

            // 使用Content::text方法将JSON转换为字符串
            let json_string = serde_json::to_string(&result).unwrap_or_default();
            Ok(vec![Content::text(json_string)])
        })
    }

    fn list_resources(&self) -> Vec<Resource> {
        vec![]
    }

    fn read_resource(
        &self,
        _uri: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, ResourceError>> + Send + 'static>> {
        Box::pin(async { Err(ResourceError::NotFound("Resource not found".into())) })
    }

    fn list_prompts(&self) -> Vec<Prompt> {
        vec![]
    }

    fn get_prompt(
        &self,
        _prompt_name: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, PromptError>> + Send + 'static>> {
        Box::pin(async { Err(PromptError::NotFound("Prompt not found".into())) })
    }
}

impl Clone for SQLiteRouter {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
        }
    }
}

/// 将JSON值转换为SQLite参数
fn json_value_to_sql(value: &Value) -> Box<dyn ToSql> {
    match value {
        Value::Null => Box::new(Option::<String>::None),
        Value::Bool(b) => Box::new(*b),
        Value::Number(n) => {
            if n.is_i64() {
                Box::new(n.as_i64().unwrap())
            } else if n.is_u64() {
                Box::new(n.as_u64().unwrap() as i64)
            } else {
                Box::new(n.as_f64().unwrap())
            }
        }
        Value::String(s) => Box::new(s.clone()),
        Value::Array(_) => Box::new(value.to_string()),
        Value::Object(_) => Box::new(value.to_string()),
    }
}

/// 从SQLite行中提取值
fn extract_row_values(row: &Row, column_names: &[String]) -> Value {
    let mut values = serde_json::Map::new();

    for (i, name) in column_names.iter().enumerate() {
        let value = match row.get_ref(i) {
            Ok(rusqlite::types::ValueRef::Null) => Value::Null,
            Ok(rusqlite::types::ValueRef::Integer(i)) => Value::Number(i.into()),
            Ok(rusqlite::types::ValueRef::Real(f)) => {
                if let Some(n) = serde_json::Number::from_f64(f) {
                    Value::Number(n)
                } else {
                    Value::Null
                }
            }
            Ok(rusqlite::types::ValueRef::Text(t)) => {
                Value::String(String::from_utf8_lossy(t).to_string())
            }
            Ok(rusqlite::types::ValueRef::Blob(b)) => Value::String(STANDARD.encode(b)),
            Err(_) => Value::Null,
        };

        values.insert(name.clone(), value);
    }

    Value::Object(values)
}
