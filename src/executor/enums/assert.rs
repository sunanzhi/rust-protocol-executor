use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssertTypeEnum {
    JsonPath,
    XmlPath,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssertOperatorEnum {
    Eq,
    NotEq,
}