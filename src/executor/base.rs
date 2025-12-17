use super::*;
use std::collections::HashMap;
use crate::middleware;

/// 基础执行器
pub struct BaseExecutor {
    middlewares: Vec<(Box<dyn middleware::Middleware>, u32)>, // (middleware, order)
}

impl BaseExecutor {
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    pub fn with_middleware(mut self, middleware: Box<dyn middleware::Middleware>, order: u32) -> Self {
        self.middlewares.push((middleware, order));
        self.middlewares.sort_by_key(|(_, order)| *order);
        self
    }

    /// 执行前置中间件
    pub async fn execute_pre_middleware(
        &self,
        context: &mut Context,
    ) -> Result<(), anyhow::Error> {
        for (middleware, _) in &self.middlewares {
            middleware.before_execute(context).await?;
        }
        Ok(())
    }

    /// 执行后置中间件
    pub async fn execute_post_middleware(
        &self,
        result: &mut ExecutionResult,
        context: &Context,
    ) -> Result<(), anyhow::Error> {
        for (middleware, _) in &self.middlewares {
            middleware.after_execute(result, context).await?;
        }
        Ok(())
    }

    /// 合并元数据
    pub fn merge_metadata(
        &self,
        base: &mut HashMap<String, String>,
        additional: &HashMap<String, String>,
    ) {
        for (key, value) in additional {
            base.insert(key.clone(), value.clone());
        }
    }
}

impl Default for BaseExecutor {
    fn default() -> Self {
        Self::new()
    }
}