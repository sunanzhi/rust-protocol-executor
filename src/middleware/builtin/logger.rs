use std::collections::HashMap;
use async_trait::async_trait;
use crate::executor::{Context, ExecutionResult};
use crate::middleware::{Middleware, MiddlewareCreator};

/// 日志中间件
#[derive(Default)]
pub struct LoggerMiddleware {
    config: Option<HashMap<String, String>>,
}

#[async_trait]
impl Middleware for LoggerMiddleware {
    async fn before(&self, context: &mut Context) -> Result<(), anyhow::Error> {
        tracing::info!(
            "Starting execution: protocol={:?}, target={}",
            "http",
            "test"
        );

        // if let Some(payload) = &context.payload {
        //     tracing::debug!("Payload: {}", payload);
        // }
        //
        // if !context.headers.is_empty() {
        //     tracing::debug!("Headers: {:?}", context.headers);
        // }

        Ok(())
    }

    async fn after(
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

impl MiddlewareCreator for LoggerMiddleware {
    fn create(&self, config: Option<HashMap<String, String>>) -> Box<dyn Middleware> {
        Box::new(LoggerMiddleware { config })
    }
}