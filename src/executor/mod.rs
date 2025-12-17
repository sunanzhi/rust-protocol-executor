mod base;
mod http;
mod websocket;

pub use base::*;
pub use http::*;
pub use websocket::*;

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use crate::{cli, config};
use crate::middleware::MiddlewareRegistry;

/// 执行上下文
#[derive(Debug, Clone)]
pub struct Context {
    pub protocol: cli::Protocol,
    pub target: String,
    pub payload: Option<String>,
    pub headers: Vec<(String, String)>,
    pub timeout: u64,
    pub retry_count: u32,
    pub metadata: HashMap<String, String>,
}

/// 执行结果
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub status: String,
    pub data: Vec<u8>,
    pub duration: std::time::Duration,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
pub trait Executor: Send + Sync {
    /// 执行主逻辑
    async fn execute(&self, context: Context) -> Result<ExecutionResult, anyhow::Error>;

    /// 获取执行器名称
    fn name(&self) -> &str;
}

/// 执行器工厂
#[derive(Clone)]
pub struct ExecutorFactory {
    middleware_registry: Arc<MiddlewareRegistry>,
    config: config::Config,
}

impl ExecutorFactory {
    pub fn new(middleware_registry: MiddlewareRegistry, config: config::Config) -> Self {
        Self {
            middleware_registry: Arc::new(middleware_registry),
            config,
        }
    }

    pub async fn create_executor(
        &self,
        protocol: &cli::Protocol,
    ) -> Result<Box<dyn Executor>, anyhow::Error> {
        // 获取该协议的中间件链配置
        let protocol_str = format!("{:?}", protocol).to_lowercase();
        let middleware_chain = self.config.get_middleware_chain(&protocol_str);

        // 创建基础执行器
        let mut base_executor = BaseExecutor::new();

        // 配置中间件
        for middleware_config in middleware_chain {
            if !middleware_config.enabled {
                continue;
            }

            match middleware_config.r#type {
                config::MiddlewareType::Builtin(ref name) => {
                    if let Some(middleware) = self.middleware_registry.get_builtin_middleware(name) {
                        base_executor = base_executor.with_middleware(
                            middleware,
                            middleware_config.order.unwrap_or(100),
                        );
                    }
                }
                config::MiddlewareType::External(ref path) => {
                    // 从路径获取中间件名称
                    let name = std::path::Path::new(path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    if let Some(middleware) = self.middleware_registry.get_external_middleware(&name) {
                        base_executor = base_executor.with_middleware(
                            Box::new(middleware),
                            middleware_config.order.unwrap_or(100),
                        );
                    }
                }
            }
        }

        match protocol {
            cli::Protocol::Http | cli::Protocol::Https => {
                let executor = HttpExecutor::new(base_executor);
                Ok(Box::new(executor))
            }
            cli::Protocol::WebSocket | cli::Protocol::Ws | cli::Protocol::Wss => {
                let executor = WebSocketExecutor::new(base_executor);
                Ok(Box::new(executor))
            }
            _ => Err(anyhow::anyhow!("Protocol not supported: {:?}", protocol)),
        }
    }
}