#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;
    use serde_json::json;
    use protocol_executor::cli::Mode::Single;
    use protocol_executor::executor::Context;
    use protocol_executor::middleware::builtin::single_transfer::SingleTransferMiddleware;
    use protocol_executor::middleware::MiddlewareCreator;

    #[tokio::test]
    async fn test_before_success() {
        // 创建临时文件
        let path =PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples").join("http.yaml");


        // 创建中间件和上下文
        let middleware = MiddlewareCreator::create(&SingleTransferMiddleware::default(), None);
        let mut context = Context {
            path,
            retry_count: 0,
            metadata: Default::default(),
            mode: Single,
            step_list: Vec::new(),
            variables: Default::default(),
        };


        // 执行测试
        let result = middleware.before(&mut context).await;
        //
        // // 验证结果
        assert!(result.is_ok());
        // assert_eq!(context.step_list.len(), 1);
    }
}