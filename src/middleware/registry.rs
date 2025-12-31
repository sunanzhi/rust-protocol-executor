use super::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 中间件注册中心
#[derive(Clone)]
pub struct MiddlewareRegistry {
    builtin_middlewares: Arc<HashMap<String, Box<dyn MiddlewareCreator>>>,
    external_middlewares: Arc<HashMap<String, ExternalMiddlewareConfig>>,
}

/// 外部中间件配置
#[derive(Debug, Clone)]
pub struct ExternalMiddlewareConfig {
    pub path: PathBuf,
    pub timeout_secs: u64,
    pub args: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub working_dir: Option<PathBuf>,
}

impl ExternalMiddlewareConfig {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            timeout_secs: 30,
            args: Vec::new(),
            env_vars: HashMap::new(),
            working_dir: None,
        }
    }
}

impl MiddlewareRegistry {
    pub fn new() -> Self {
        Self {
            builtin_middlewares: Arc::new(HashMap::new()),
            external_middlewares: Arc::new(HashMap::new()),
        }
    }

    /// 注册内置中间件
    pub fn register_builtin_middlewares(&mut self) {
        let mut builtin_map = HashMap::new();

        builtin_map.insert(
            "logger".to_string(),
            Box::new(builtin::logger::LoggerMiddleware::default()) as Box<dyn MiddlewareCreator>
        );

        builtin_map.insert(
            "validator".to_string(),
            Box::new(builtin::validator::ValidatorMiddleware::default()) as Box<dyn MiddlewareCreator>
        );

        builtin_map.insert(
            "metrics".to_string(),
            Box::new(builtin::metrics::MetricsMiddleware::default()) as Box<dyn MiddlewareCreator>
        );

        self.builtin_middlewares = Arc::new(builtin_map);
    }

    /// 注册外部中间件目录
    pub fn register_external_middlewares<P: AsRef<Path>>(
        &mut self,
        dir: P,
    ) -> Result<(), anyhow::Error> {
        let dir = dir.as_ref();
        if !dir.exists() {
            tracing::warn!("Middleware directory does not exist: {:?}", dir);
            return Ok(());
        }

        let mut external_map = HashMap::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && is_executable(&path) {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let config = ExternalMiddlewareConfig::new(path.clone());
                external_map.insert(name, config);

                tracing::info!("Registered external middleware: {:?}", path);
            }
        }

        self.external_middlewares = Arc::new(external_map);
        Ok(())
    }

    /// 注册单个外部中间件
    pub fn register_external_middleware<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<(), anyhow::Error> {
        let path = path.as_ref();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut external_map = HashMap::clone(&self.external_middlewares);
        let config = ExternalMiddlewareConfig::new(path.to_path_buf());
        external_map.insert(name, config);

        self.external_middlewares = Arc::new(external_map);
        Ok(())
    }

    /// 获取内置中间件
    pub fn get_builtin_middleware(&self, name: &str) -> Option<Box<dyn Middleware>> {
        self.builtin_middlewares
            .get(name)
            .map(|creator| creator.create(None))
    }

    /// 获取外部中间件
    pub fn get_external_middleware(&self, name: &str) -> Option<ExternalMiddleware> {
        self.external_middlewares
            .get(name)
            .map(|config| ExternalMiddleware::new(config.path.clone()))
    }

    /// 根据类型获取中间件
    pub fn get_middleware(&self, middleware_type: &MiddlewareType) -> Option<Box<dyn Middleware>> {
        match middleware_type {
            MiddlewareType::Builtin(name) => self.get_builtin_middleware(name),
            MiddlewareType::External(name) => {
                self.get_external_middleware(name.to_str()?)
                    .map(|m| Box::new(m) as Box<dyn Middleware>)
            }
        }
    }

    /// 获取所有中间件名称
    pub fn get_middleware_names(&self) -> Vec<String> {
        let mut names = Vec::new();

        // 添加内置中间件名称
        for name in self.builtin_middlewares.keys() {
            names.push(format!("builtin:{}", name));
        }

        // 添加外部中间件名称
        for name in self.external_middlewares.keys() {
            names.push(format!("external:{}", name));
        }

        names
    }
}

impl Default for MiddlewareRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register_builtin_middlewares();
        registry
    }
}

fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = path.metadata() {
            return metadata.permissions().mode() & 0o111 != 0;
        }
    }

    #[cfg(windows)]
    {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            return ext_str == "exe" || ext_str == "bat" || ext_str == "cmd" || ext_str == "ps1";
        }
    }

    true
}