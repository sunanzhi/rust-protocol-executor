use anyhow::Error;
use async_trait::async_trait;
use crate::executor::{BaseExecutor, Context, ExecutionResult};
use crate::executor::mode::Mode;

pub struct SingleMode {
    base: BaseExecutor,
}

impl SingleMode {
    pub fn new(base: BaseExecutor,) -> Self {
        SingleMode {
            base,
        }
    }
}

#[async_trait]
impl Mode for SingleMode {
    async fn execute(&self, mut context: Context) -> Result<ExecutionResult, Error> {

        // 执行前置中间件
        self.base.execute_pre_middleware(&mut context).await?;


        // 构建执行结果
        let mut result = ExecutionResult {
            success: true,
            status: "500".to_string(),
            data: Default::default(),
            duration: Default::default(),
            metadata: Default::default(),
        };


        // 执行后置中间件
        self.base
            .execute_post_middleware(&mut result, &context)
            .await?;

        Ok(result)
    }

    fn mode(&self) -> &str {
        "single"
    }
}