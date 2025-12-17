mod cli;
mod config;
mod executor;
mod middleware;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use config::Config;
use executor::ExecutorFactory;
use middleware::MiddlewareRegistry;
use std::path::PathBuf;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 解析命令行参数
    let cli = Cli::parse();

    // 加载配置文件
    let config = if let Some(config_path) = &cli.config {
        Config::from_file(config_path)?
    } else {
        Config::default()
    };

    // 创建中间件注册中心
    let mut registry = MiddlewareRegistry::new();

    // 注册内置中间件
    registry.register_builtin_middlewares();

    // 注册外部中间件
    if let Some(external_dir) = &config.external_middleware_dir {
        registry.register_external_middlewares(external_dir)?;
    }

    // 注册命令行指定的中间件
    if let Some(middleware_paths) = &cli.middleware {
        for path in middleware_paths {
            registry.register_external_middleware(path)?;
        }
    }

    // 创建执行器工厂
    let factory = ExecutorFactory::new(registry, config);

    // 创建指定协议的执行器
    let executor = factory.create_executor(&cli.protocol).await?;

    // 准备执行上下文
    let context = executor::Context {
        protocol: cli.protocol.clone(),
        target: cli.target.clone(),
        payload: cli.payload.clone(),
        headers: cli.headers.clone().unwrap_or_default(),
        timeout: cli.timeout,
        retry_count: cli.retry_count,
        metadata: Default::default(),
    };

    // 执行
    let result = executor.execute(context).await?;

    // 输出结果
    if cli.verbose {
        println!("Execution completed:");
        println!("Status: {:?}", result.status);
        println!("Duration: {:?}", result.duration);
        println!("Data: {}", String::from_utf8_lossy(&result.data));
        if !result.metadata.is_empty() {
            println!("Metadata: {:?}", result.metadata);
        }
    } else {
        println!("{}", String::from_utf8_lossy(&result.data));
    }

    Ok(())
}