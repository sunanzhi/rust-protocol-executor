mod base;
mod http;
mod websocket;
pub mod mode;

pub use base::*;
pub use http::*;
pub use websocket::*;

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use crate::{cli, config};
use crate::executor::mode::ProtocolModel;
use crate::middleware::MiddlewareRegistry;

/// 执行上下文
#[derive(Debug)]
pub struct Context {
    pub mode: cli::Mode,
    pub path: PathBuf,
    pub retry_count: u32,
    pub metadata: HashMap<String, String>,
    pub variables: HashMap<String, String>,
    pub step_list: Vec<ProtocolModel>,
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
        protocol: &mode::Protocol,
    ) -> Result<Box<dyn Executor>, anyhow::Error> {
        // 创建基础执行器 @todo 继承模式执行器
        let base_executor = BaseExecutor::new();


        match protocol {
            mode::Protocol::Http | mode::Protocol::Https => {
                let executor = HttpExecutor::new(base_executor);
                Ok(Box::new(executor))
            }
            mode::Protocol::WebSocket | mode::Protocol::Ws | mode::Protocol::Wss => {
                let executor = WebSocketExecutor::new(base_executor);
                Ok(Box::new(executor))
            }
            _ => Err(anyhow::anyhow!("Protocol not supported: {:?}", protocol)),
        }
    }
}