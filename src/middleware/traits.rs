use crate::executor::{Context, ExecutionResult};
use async_trait::async_trait;
use std::collections::HashMap;

/// 中间件特征
#[async_trait]
pub trait Middleware: Send + Sync {
    /// 在执行前调用，可以修改上下文
    async fn before(&self, context: &mut Context) -> Result<(), anyhow::Error>;

    /// 在执行后调用，可以修改结果
    async fn after(
        &self,
        result: &mut ExecutionResult,
        context: &Context,
    ) -> Result<(), anyhow::Error>;

    /// 获取中间件名称
    fn name(&self) -> &str;

    /// 获取中间件配置
    fn config(&self) -> Option<&HashMap<String, String>>;
}

/// 中间件创建器特征
pub trait MiddlewareCreator {
    fn create(&self, config: Option<HashMap<String, String>>) -> Box<dyn Middleware>;
}