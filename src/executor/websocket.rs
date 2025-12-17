use super::*;
use futures::{SinkExt, StreamExt};
use std::time::Instant;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Utf8Bytes;

pub struct WebSocketExecutor {
    base: BaseExecutor,
}

impl WebSocketExecutor {
    pub fn new(base: BaseExecutor) -> Self {
        Self { base }
    }

    async fn connect_websocket(
        &self,
        url: &str,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, anyhow::Error> {
        let (ws_stream, _) = connect_async(url).await?;
        Ok(ws_stream)
    }
}

#[async_trait]
impl Executor for WebSocketExecutor {
    async fn execute(&self, mut context: Context) -> Result<ExecutionResult, anyhow::Error> {
        let start_time = Instant::now();

        // 执行前置中间件
        self.base.execute_pre_middleware(&mut context).await?;

        // 连接WebSocket
        let mut ws_stream = self.connect_websocket(&context.target).await?;

        // 发送消息（如果有payload）
        if let Some(payload) = &context.payload {
            ws_stream
                .send(Message::Text(Utf8Bytes::from(payload.clone())))
                .await
                .map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))?;
        }

        // 接收消息
        let mut received_data = Vec::new();
        let mut metadata = HashMap::new();
        metadata.insert("protocol".to_string(), "websocket".to_string());

        if let Some(message) = ws_stream.next().await {
            match message {
                Ok(msg) => {
                    match msg {
                        Message::Text(text) => {
                            received_data.extend_from_slice(text.as_bytes());
                        }
                        Message::Binary(data) => {
                            received_data = Vec::from(data);
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("WebSocket error: {}", e));
                }
            }
        }

        let duration = start_time.elapsed();

        // 构建执行结果
        let mut result = ExecutionResult {
            success: true,
            status: "connected".to_string(),
            data: received_data,
            duration,
            metadata,
        };

        // 执行后置中间件
        self.base
            .execute_post_middleware(&mut result, &context)
            .await?;

        Ok(result)
    }

    fn name(&self) -> &str {
        "websocket_executor"
    }
}