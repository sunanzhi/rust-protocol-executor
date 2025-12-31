use crate::executor::mode::Mode;
use crate::executor::{BaseExecutor, Context, ExecutionResult};
use anyhow::Error;
use async_trait::async_trait;

pub struct WorkflowMode {
    base: BaseExecutor,
}

impl WorkflowMode {
    pub fn new(base: BaseExecutor,) -> Self {
        WorkflowMode {
            base,
        }
    }
}

#[async_trait]
impl Mode for WorkflowMode {
    async fn execute(&self, context: Context) -> Result<ExecutionResult, Error> {
        // 构建执行结果
        let result = ExecutionResult {
            success: true,
            status: "500".to_string(),
            data: Default::default(),
            duration: Default::default(),
            metadata: Default::default(),
        };


        Ok(result)
    }

    fn mode(&self) -> &str {
        "workflow"
    }
}