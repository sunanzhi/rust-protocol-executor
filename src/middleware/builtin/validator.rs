use std::collections::HashMap;
use async_trait::async_trait;
use crate::executor::{Context, ExecutionResult};
use crate::middleware::{Middleware, MiddlewareCreator};

/// 验证器中间件
#[derive(Default)]
pub struct ValidatorMiddleware {
    config: Option<HashMap<String, String>>,
}

#[async_trait]
impl Middleware for ValidatorMiddleware {
    async fn before(&self, context: &mut Context) -> Result<(), anyhow::Error> {
        // 验证URL格式
        // if context.target.is_empty() {
        //     return Err(anyhow::anyhow!("Target URL cannot be empty"));
        // }
        // 
        // // 验证协议支持
        // match context.protocol {
        //     cli::Protocol::Http | cli::Protocol::Https => {
        //         if !context.target.starts_with("http") {
        //             return Err(anyhow::anyhow!("Invalid HTTP URL: {}", context.target));
        //         }
        //     }
        //     cli::Protocol::WebSocket | cli::Protocol::Ws | cli::Protocol::Wss => {
        //         if !context.target.starts_with("ws") {
        //             return Err(anyhow::anyhow!("Invalid WebSocket URL: {}", context.target));
        //         }
        //     }
        //     _ => {}
        // }
        // 
        // // 验证超时设置
        // if context.timeout == 0 {
        //     context.timeout = 30; // 设置默认超时
        // }

        Ok(())
    }

    async fn after(
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

impl MiddlewareCreator for ValidatorMiddleware {
    fn create(&self, config: Option<HashMap<String, String>>) -> Box<dyn Middleware> {
        Box::new(ValidatorMiddleware { config })
    }
}
