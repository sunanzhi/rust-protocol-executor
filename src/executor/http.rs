use super::*;
use reqwest::Client;
use std::time::Instant;

pub struct HttpExecutor {
    base: BaseExecutor,
    client: Client,
}

impl HttpExecutor {
    pub fn new(base: BaseExecutor) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { base, client }
    }
}

#[async_trait]
impl Executor for HttpExecutor {
    async fn execute(&self, mut context: Context) -> Result<ExecutionResult, anyhow::Error> {
        let start_time = Instant::now();

        // 执行前置中间件
        self.base.execute_pre_middleware(&mut context).await?;

        // 构建HTTP请求
        let mut request_builder = self.client.request(reqwest::Method::GET, "x");

        // 添加headers
        // for (key, value) in &context.headers {
        //     request_builder = request_builder.header(key, value);
        // }
        // 
        // // 添加payload（如果有）
        // if let Some(payload) = &context.payload {
        //     request_builder = request_builder.body(payload.clone());
        // }

        // 发送请求
        let response = request_builder.send().await?;
        let status = response.status();
        let headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        // 读取响应体
        let data = response.bytes().await?.to_vec();

        let duration = start_time.elapsed();

        // 构建执行结果
        let mut result = ExecutionResult {
            success: status.is_success(),
            status: status.to_string(),
            data,
            duration,
            metadata: headers,
        };

        // 执行后置中间件
        self.base
            .execute_post_middleware(&mut result, &context)
            .await?;

        Ok(result)
    }

    fn name(&self) -> &str {
        "http_executor"
    }
}