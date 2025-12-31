use crate::executor::{Context, ExecutionResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use base64::Engine;
use base64::engine::general_purpose;
use tokio::io::{AsyncWriteExt};
use tokio::process::Command;

/// 外部中间件通信协议
#[derive(Debug, Serialize, Deserialize)]
pub struct MiddlewareRequest {
    pub action: String, // "before" 或 "after"
    pub execution_id: String,
    pub protocol: String,
    pub target: String,
    pub payload: Option<String>,
    pub headers: Vec<(String, String)>,
    pub metadata: serde_json::Value,
    pub config: Option<serde_json::Value>,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MiddlewareResponse {
    pub action: String,
    pub execution_id: String,
    pub should_continue: bool,
    pub modified_payload: Option<String>,
    pub modified_headers: Option<Vec<(String, String)>>,
    pub modified_metadata: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub metrics: Option<serde_json::Value>,
}

/// 外部中间件错误类型
#[derive(thiserror::Error, Debug)]
pub enum ExternalMiddlewareError {
    #[error("Middleware execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Middleware timeout after {0} seconds")]
    Timeout(u64),

    #[error("Failed to parse middleware response: {0}")]
    ParseError(String),

    #[error("Middleware rejected the request: {0}")]
    Rejected(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// 外部中间件配置
#[derive(Debug, Clone)]
pub struct ExternalMiddlewareConfig {
    pub path: PathBuf,
    pub timeout_secs: u64,
    pub args: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub working_dir: Option<PathBuf>,
}

impl Default for ExternalMiddlewareConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            timeout_secs: 30,
            args: Vec::new(),
            env_vars: HashMap::new(),
            working_dir: None,
        }
    }
}

/// 外部中间件实例
pub struct ExternalMiddleware {
    config: ExternalMiddlewareConfig,
    name: String,
}

impl ExternalMiddleware {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path_buf = path.as_ref().to_path_buf();
        let name = path_buf
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Self {
            config: ExternalMiddlewareConfig {
                path: path_buf,
                ..Default::default()
            },
            name,
        }
    }

    pub fn with_config(mut self, config: ExternalMiddlewareConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.config.timeout_secs = timeout_secs;
        self
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.config.args = args;
        self
    }

    /// 调用外部中间件
    async fn call_middleware(
        &self,
        request: &MiddlewareRequest,
    ) -> Result<MiddlewareResponse, ExternalMiddlewareError> {
        // 准备执行命令
        let mut cmd = Command::new(&self.config.path);

        // 添加参数
        if !self.config.args.is_empty() {
            cmd.args(&self.config.args);
        }

        // 设置环境变量
        for (key, value) in &self.config.env_vars {
            cmd.env(key, value);
        }

        // 设置工作目录
        if let Some(working_dir) = &self.config.working_dir {
            cmd.current_dir(working_dir);
        }

        // 设置标准输入输出
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // 启动进程
        let mut child = cmd.spawn()?;

        // 准备请求数据
        let request_json = serde_json::to_string(request)?;

        // 写入标准输入
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(request_json.as_bytes()).await?;
            stdin.flush().await?;
        }

        // 等待进程完成（带超时）
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(self.config.timeout_secs),
            child.wait_with_output(),
        )
            .await
            .map_err(|_| ExternalMiddlewareError::Timeout(self.config.timeout_secs))?
            .map_err(ExternalMiddlewareError::Io)?;

        // 检查退出状态
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ExternalMiddlewareError::ExecutionFailed(stderr.to_string()));
        }

        // 解析响应
        let response_json = String::from_utf8_lossy(&output.stdout);
        let response: MiddlewareResponse = serde_json::from_str(&response_json)
            .map_err(|e| ExternalMiddlewareError::ParseError(e.to_string()))?;

        // 验证执行ID匹配
        if response.execution_id != request.execution_id {
            return Err(ExternalMiddlewareError::ParseError(
                "Execution ID mismatch".to_string(),
            ));
        }

        Ok(response)
    }
}

#[async_trait]
impl super::Middleware for ExternalMiddleware {
    async fn before(&self, context: &mut Context) -> Result<(), anyhow::Error> {
        let execution_id = uuid::Uuid::default().to_string();

        // 构建请求
        let request = MiddlewareRequest {
            action: "before".to_string(),
            execution_id: execution_id.clone(),
            protocol: Default::default(),
            target: Default::default(),
            payload: Default::default(),
            headers: Default::default(),
            metadata: serde_json::to_value(&context.metadata)?,
            config: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // 调用中间件
        let response = self.call_middleware(&request).await?;

        // 检查是否继续
        if !response.should_continue {
            return Err(anyhow::anyhow!(
                "Middleware rejected request: {}",
                response.error_message.unwrap_or_default()
            ));
        }

        // 应用修改
        // if let Some(modified_payload) = response.modified_payload {
        //     context.payload = Some(modified_payload);
        // }

        // if let Some(modified_headers) = response.modified_headers {
        //     context.headers = modified_headers;
        // }

        if let Some(modified_metadata) = response.modified_metadata {
            if let Ok(metadata_map) = serde_json::from_value::<HashMap<String, String>>(modified_metadata) {
                for (key, value) in metadata_map {
                    context.metadata.insert(key, value);
                }
            }
        }

        // 存储中间件执行ID
        context
            .metadata
            .insert(format!("{}_execution_id", self.name), execution_id);

        Ok(())
    }

    async fn after(
        &self,
        result: &mut ExecutionResult,
        context: &Context,
    ) -> Result<(), anyhow::Error> {
        let execution_id = uuid::Uuid::default().to_string();

        // 准备响应数据
        let response_data = if result.data.is_empty() {
            None
        } else {
            Some(general_purpose::STANDARD_NO_PAD.encode(&result.data))
        };

        // 构建请求
        let request = MiddlewareRequest {
            action: "after".to_string(),
            execution_id: execution_id.clone(),
            protocol: Default::default(),
            target: Default::default(),
            payload: response_data,
            headers: Default::default(),
            metadata: serde_json::json!({
                "original_metadata": context.metadata,
                "execution_result": {
                    "success": result.success,
                    "status": result.status.clone(),
                    "duration_ms": result.duration.as_millis(),
                }
            }),
            config: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // 调用中间件
        let response = self.call_middleware(&request).await?;

        // 应用修改
        if let Some(modified_payload) = response.modified_payload {
            let encoded = general_purpose::STANDARD_NO_PAD.encode(modified_payload);
            result.data = Vec::from(encoded);
        }

        if let Some(modified_metadata) = response.modified_metadata {
            if let Ok(metadata_map) = serde_json::from_value::<HashMap<String, String>>(modified_metadata) {
                for (key, value) in metadata_map {
                    result.metadata.insert(key, value);
                }
            }
        }

        // 添加中间件指标
        if let Some(metrics) = response.metrics {
            result
                .metadata
                .insert(format!("{}_metrics", self.name), metrics.to_string());
        }

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn config(&self) -> Option<&HashMap<String, String>> {
        None // 外部中间件配置不暴露给内部
    }
}