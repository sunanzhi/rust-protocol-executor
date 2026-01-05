use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamTypeEnum {
    String,
    Integer,
    Number,
    Boolean,
    Binary,
    Byte,
    Array,
    Object,
}