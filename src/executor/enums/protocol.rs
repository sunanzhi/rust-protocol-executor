use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolEnum {
    Http,
    Https,
    WebSocket,
    Ws,
    Wss,
    Tcp,
    Udp,
}