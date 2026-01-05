mod http;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use crate::executor::enums::assert::{AssertOperatorEnum, AssertTypeEnum};
use crate::executor::enums::param_type::ParamTypeEnum;
use crate::executor::enums::protocol::ProtocolEnum;
use crate::executor::protocol::http::HttpProtocolModel;

#[derive(Debug, Serialize, Deserialize)]
pub struct BaseProtocol {
    pub variables: HashMap<String, Value>,
    pub meta: HashMap<String, Value>,
    pub protocol: ProtocolEnum,
    pub data: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParamDTO {
    name: String,
    value: Value,
    r#type: ParamTypeEnum
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssertDTO {
    r#type: AssertTypeEnum,
    type_value: Value,
    value: Value,
    operator: AssertOperatorEnum,
}

// 使用枚举封装所有可能类型
#[derive(Debug)]
enum ProtocolDataType {
    Http(HttpProtocolModel),
    Unknown(Value),
}


impl BaseProtocol {
    pub fn parse_data(&self) -> Result<ProtocolDataType, anyhow::Error> {
        match self.protocol {
            ProtocolEnum::Http | ProtocolEnum::Https => {
                let data: HttpProtocolModel = match serde_yaml::from_value(self.data.clone()) {
                    Ok(data) => data,
                    Err(error) => {
                        tracing::error!("{}", error);
                        return Err(anyhow::anyhow!("Failed to parse YAML: {}", error));
                    }
                };
                Ok(ProtocolDataType::Http(data))
            }
            _ => Ok(ProtocolDataType::Unknown(self.data.clone())),
        }
    }
}