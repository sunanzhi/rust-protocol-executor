use std::collections::HashMap;
use std::fs;
use async_trait::async_trait;
use crate::executor::{Context, ExecutionResult};
use crate::middleware::{Middleware, MiddlewareCreator};

/// 单一步骤模式转换
#[derive(Default)]
pub struct SingleTransferMiddleware {
    config: Option<HashMap<String, String>>,
}

#[async_trait]
impl Middleware for SingleTransferMiddleware {
    async fn before(&self, context: &mut Context) -> Result<(), anyhow::Error> {
        tracing::info!(
            "Starting transfer: mode={:?}",
            context.mode
        );
        let single_content = fs::read_to_string(&context.path)?.to_string();


        // 解析请求
        let request: crate::executor::protocol::BaseProtocol = match serde_yaml::from_str(&single_content) {
            Ok(req) => req,
            Err(e) => {
                tracing::error!("Failed to parse request: {:?}", e);
                return Err(anyhow::anyhow!("Failed to parse request: {:?}", e));
            }
        };

        context.step_list.push(request);

        Ok(())
    }

    async fn after(
        &self,
        result: &mut ExecutionResult,
        context: &Context,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn name(&self) -> &str {
        "single transfer"
    }

    fn config(&self) -> Option<&HashMap<String, String>> {
        self.config.as_ref()
    }
}

impl MiddlewareCreator for SingleTransferMiddleware {
    fn create(&self, config: Option<HashMap<String, String>>) -> Box<dyn Middleware> {
        Box::new(SingleTransferMiddleware { config })
    }
}