mod common;
#[cfg(test)]
mod middleware {
    use super::*;
    use std::path::PathBuf;
    use protocol_executor::cli::Mode::Single;
    use protocol_executor::executor::Context;
    use protocol_executor::middleware::builtin::single_transfer::SingleTransferMiddleware;
    use protocol_executor::middleware::MiddlewareCreator;

    #[tokio::test]
    async fn test_before_success() {
        common::init();
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

        tracing::info!("{}", serde_json::to_string_pretty(&context.step_list[0]).unwrap());

        // 验证结果
        assert!(result.is_ok());
        assert_eq!(context.step_list.len(), 1);
    }
}