mod traits;
mod registry;
mod builtin;
mod external;

pub use traits::*;
pub use registry::*;
pub use builtin::*;
pub use external::*;

use std::collections::HashMap;
use std::path::PathBuf;

/// 中间件类型
#[derive(Debug, Clone)]
pub enum MiddlewareType {
    Builtin(String),
    External(PathBuf),
}