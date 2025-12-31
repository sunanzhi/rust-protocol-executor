mod single;
mod workflow;

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use crate::{cli, config};
use crate::executor::{BaseExecutor, Context, ExecutionResult};
use crate::executor::mode::single::SingleMode;
use crate::executor::mode::workflow::WorkflowMode;
use crate::middleware::MiddlewareRegistry;

/// 模式解析结果
#[derive(Debug, Clone)]
pub struct ModeParseResult {
    pub variables: HashMap<String, String>,
    pub step_list: Vec<ProtocolModel>,
}

#[derive(Debug, Clone)]
pub struct ProtocolModel {
    id: String,
    name: String,
    description: String,
    version: String,
    author: String,
    protocol: Protocol,
    request: RequestModel
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestModel {

}

/// http请求模型
#[derive(Debug, Serialize, Deserialize)]
pub struct HttpRequestModel {
    url: String,
    method: String,
    headers: HashMap<String, String>,
    body: String,
    query: HashMap<String, String>,
    path: HashMap<String, String>,
    params: HashMap<String, Param>,
}

/// 参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    name: String,
    value: String,
    r#type: ParamTypeEnum,
}

/// 参数类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamTypeEnum {
    String,
    Integer,
    Number,
    Boolean,
    Binary,
    Byte,
    Array,
    Object,
}

trait RequestModelTrait {}

impl RequestModel {}

impl RequestModelTrait for HttpRequestModel {

}

#[derive(Clone, Debug, ValueEnum)]
pub enum Protocol {
    Http,
    Https,
    WebSocket,
    Ws,
    Wss,
    Tcp,
    Udp,
}

#[async_trait]
pub trait Mode: Send + Sync {
    /// 执行主逻辑
    async fn execute(&self, context: Context) -> Result<ExecutionResult, anyhow::Error>;

    /// 获取模式名称
    fn mode(&self) -> &str;
}

/// Mode Factory
#[derive(Clone)]
pub struct ModeFactory {
    middleware_registry: Arc<MiddlewareRegistry>,
    config: config::Config,
}

impl ModeFactory {
    pub fn new(middleware_registry: MiddlewareRegistry, config: config::Config) -> Self {
        Self {
            middleware_registry: Arc::new(middleware_registry),
            config,
        }
    }

    pub async fn create_mode(&self, mode: &cli::Mode) -> Result<Box<dyn Mode>, anyhow::Error> {
        // 获取该协议的中间件链配置
        let mode_str = format!("{:?}", mode).to_lowercase();
        // 获取该协议的中间件链配置
        let middleware_chain = self.config.get_middleware_chain(&mode_str);

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

        match mode {
            cli::Mode::Single => {
                Ok(Box::new(SingleMode::new(base_executor)))
            }
            cli::Mode::Workflow => {
                Ok(Box::new(WorkflowMode::new(base_executor)))
            }
        }
    }
}