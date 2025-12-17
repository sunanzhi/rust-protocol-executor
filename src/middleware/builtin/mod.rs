use crate::executor::{Context, ExecutionResult};
use async_trait::async_trait;
use std::collections::HashMap;
use crate::cli;

/// 日志中间件
#[derive(Default)]
pub struct LoggerMiddleware {
    config: Option<HashMap<String, String>>,
}

#[async_trait]
impl super::Middleware for LoggerMiddleware {
    async fn before_execute(&self, context: &mut Context) -> Result<(), anyhow::Error> {
        tracing::info!(
            "Starting execution: protocol={:?}, target={}",
            context.protocol,
            context.target
        );

        if let Some(payload) = &context.payload {
            tracing::debug!("Payload: {}", payload);
        }

        if !context.headers.is_empty() {
            tracing::debug!("Headers: {:?}", context.headers);
        }

        Ok(())
    }

    async fn after_execute(
        &self,
        result: &mut ExecutionResult,
        context: &Context,
    ) -> Result<(), anyhow::Error> {
        tracing::info!(
            "Execution completed: success={}, duration={:?}, status={}",
            result.success,
            result.duration,
            result.status
        );

        if !result.success {
            tracing::error!("Execution failed with status: {}", result.status);
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "logger"
    }

    fn config(&self) -> Option<&HashMap<String, String>> {
        self.config.as_ref()
    }
}

impl super::MiddlewareCreator for LoggerMiddleware {
    fn create(&self, config: Option<HashMap<String, String>>) -> Box<dyn super::Middleware> {
        Box::new(LoggerMiddleware { config })
    }
}

/// 验证器中间件
#[derive(Default)]
pub struct ValidatorMiddleware {
    config: Option<HashMap<String, String>>,
}

#[async_trait]
impl super::Middleware for ValidatorMiddleware {
    async fn before_execute(&self, context: &mut Context) -> Result<(), anyhow::Error> {
        // 验证URL格式
        if context.target.is_empty() {
            return Err(anyhow::anyhow!("Target URL cannot be empty"));
        }

        // 验证协议支持
        match context.protocol {
            cli::Protocol::Http | cli::Protocol::Https => {
                if !context.target.starts_with("http") {
                    return Err(anyhow::anyhow!("Invalid HTTP URL: {}", context.target));
                }
            }
            cli::Protocol::WebSocket | cli::Protocol::Ws | cli::Protocol::Wss => {
                if !context.target.starts_with("ws") {
                    return Err(anyhow::anyhow!("Invalid WebSocket URL: {}", context.target));
                }
            }
            _ => {}
        }

        // 验证超时设置
        if context.timeout == 0 {
            context.timeout = 30; // 设置默认超时
        }

        Ok(())
    }

    async fn after_execute(
        &self,
        result: &mut ExecutionResult,
        _context: &Context,
    ) -> Result<(), anyhow::Error> {
        // 可以在这里验证响应结果
        if result.data.len() > 10 * 1024 * 1024 {
            // 10MB限制
            tracing::warn!("Response size exceeds 10MB");
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "validator"
    }

    fn config(&self) -> Option<&HashMap<String, String>> {
        self.config.as_ref()
    }
}

impl super::MiddlewareCreator for ValidatorMiddleware {
    fn create(&self, config: Option<HashMap<String, String>>) -> Box<dyn super::Middleware> {
        Box::new(ValidatorMiddleware { config })
    }
}

/// 指标收集中间件
#[derive(Default)]
pub struct MetricsMiddleware {
    config: Option<HashMap<String, String>>,
}

#[async_trait]
impl super::Middleware for MetricsMiddleware {
    async fn before_execute(&self, context: &mut Context) -> Result<(), anyhow::Error> {
        // 记录开始时间
        context
            .metadata
            .insert("start_time".to_string(), chrono::Utc::now().to_rfc3339());

        // 生成请求ID
        let request_id = uuid::Uuid::default().to_string();
        context
            .metadata
            .insert("request_id".to_string(), request_id.clone());

        tracing::info!("Request started: id={}", request_id);

        Ok(())
    }

    async fn after_execute(
        &self,
        result: &mut ExecutionResult,
        context: &Context,
    ) -> Result<(), anyhow::Error> {
        // 计算总耗时
        if let Some(start_time_str) = context.metadata.get("start_time") {
            if let Ok(start_time) = chrono::DateTime::parse_from_rfc3339(start_time_str) {
                let end_time = chrono::Utc::now();
                let duration = end_time.signed_duration_since(start_time);

                result.metadata.insert(
                    "total_duration_ms".to_string(),
                    duration.num_milliseconds().to_string(),
                );
            }
        }

        // 记录响应大小
        result.metadata.insert(
            "response_size_bytes".to_string(),
            result.data.len().to_string(),
        );

        // 记录成功率
        result.metadata.insert(
            "success".to_string(),
            result.success.to_string(),
        );

        tracing::info!(
            "Metrics collected: request_id={}, success={}, size={} bytes",
            context.metadata.get("request_id").unwrap_or(&"unknown".to_string()),
            result.success,
            result.data.len()
        );

        Ok(())
    }

    fn name(&self) -> &str {
        "metrics"
    }

    fn config(&self) -> Option<&HashMap<String, String>> {
        self.config.as_ref()
    }
}

impl super::MiddlewareCreator for MetricsMiddleware {
    fn create(&self, config: Option<HashMap<String, String>>) -> Box<dyn super::Middleware> {
        Box::new(MetricsMiddleware { config })
    }
}