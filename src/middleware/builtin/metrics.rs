use std::collections::HashMap;
use async_trait::async_trait;
use crate::executor::{Context, ExecutionResult};
use crate::middleware::{Middleware, MiddlewareCreator};

/// 指标收集中间件
#[derive(Default)]
pub struct MetricsMiddleware {
    config: Option<HashMap<String, String>>,
}

#[async_trait]
impl Middleware for MetricsMiddleware {
    async fn before(&self, context: &mut Context) -> Result<(), anyhow::Error> {
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

    async fn after(
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

impl MiddlewareCreator for MetricsMiddleware {
    fn create(&self, config: Option<HashMap<String, String>>) -> Box<dyn Middleware> {
        Box::new(MetricsMiddleware { config })
    }
}