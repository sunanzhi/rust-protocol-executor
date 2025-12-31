use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io;
use std::io::Read;

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    action: String,
    execution_id: String,
    protocol: String,
    target: String,
    payload: Option<String>,
    headers: Vec<(String, String)>,
    metadata: Value,
    config: Option<Value>,
    timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    action: String,
    execution_id: String,
    should_continue: bool,
    modified_payload: Option<String>,
    modified_headers: Option<Vec<(String, String)>>,
    modified_metadata: Option<Value>,
    error_message: Option<String>,
    metrics: Option<Value>,
}

fn format_json_response(payload: &str) -> String {
    // 尝试格式化JSON响应
    match serde_json::from_str::<Value>(payload) {
        Ok(parsed) => {
            // 添加格式化信息
            let mut formatted = parsed.clone();
            if let Some(obj) = formatted.as_object_mut() {
                obj.insert(
                    "_formatted".to_string(),
                    Value::String("pretty-printed-by-middleware".to_string()),
                );
                obj.insert(
                    "_format_timestamp".to_string(),
                    Value::String(chrono::Utc::now().to_rfc3339()),
                );
            }
            serde_json::to_string_pretty(&formatted).unwrap_or(payload.to_string())
        }
        Err(_) => payload.to_string(), // 不是JSON，保持原样
    }
}

fn main() {
    // 读取标准输入
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read from stdin");

    // 解析请求
    let request: Request = match serde_json::from_str(&input) {
        Ok(req) => req,
        Err(e) => {
            let error_response = Response {
                action: "error".to_string(),
                execution_id: "unknown".to_string(),
                should_continue: false,
                modified_payload: None,
                modified_headers: None,
                modified_metadata: None,
                error_message: Some(format!("Failed to parse request: {}", e)),
                metrics: None,
            };
            println!("{}", serde_json::to_string(&error_response).unwrap());
            std::process::exit(1);
        }
    };

    let response = match request.action.as_str() {
        "before" => {
            // 前置处理：可以修改请求
            Response {
                action: "before".to_string(),
                execution_id: request.execution_id,
                should_continue: true,
                modified_payload: None,
                modified_headers: Some(vec![(
                    "X-Formatted-By".to_string(),
                    "response_formatter".to_string(),
                )]),
                modified_metadata: None,
                error_message: None,
                metrics: Some(serde_json::json!({
                    "formatter_version": "1.0",
                    "action": "before"
                })),
            }
        }
        "after" => {
            // 后置处理：格式化响应
            let modified_payload = request.payload.map(|p| {
                if p.len() > 1024 * 1024 {
                    // 如果响应太大，不进行格式化
                    p
                } else {
                    format_json_response(&p)
                }
            });

            Response {
                action: "after".to_string(),
                execution_id: request.execution_id,
                should_continue: true,
                modified_payload,
                modified_headers: None,
                modified_metadata: None,
                error_message: None,
                metrics: Some(serde_json::json!({
                    "formatter_version": "1.0",
                    "action": "after",
                    "processed": true
                })),
            }
        }
        _ => Response {
            action: request.action,
            execution_id: request.execution_id,
            should_continue: false,
            modified_payload: None,
            modified_headers: None,
            modified_metadata: None,
            error_message: Some(format!("Unknown action: {}", "")),
            metrics: None,
        },
    };

    // 输出响应
    println!("{}", serde_json::to_string(&response).unwrap());
}