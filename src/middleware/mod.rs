mod traits;
pub mod registry;
pub mod builtin;
mod external;

pub use external::*;
pub use registry::*;
pub use traits::*;

use std::path::PathBuf;

/// 中间件类型
#[derive(Debug, Clone)]
pub enum MiddlewareType {
    Builtin(String),
    External(PathBuf),
}