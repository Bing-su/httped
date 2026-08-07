use salvo::prelude::*;
use salvo::websocket::WebSocketUpgrade;
use tracing;

#[endpoint(
    tags("Websocket"),
    status_codes(200, 400, 500),
    description = "Echoes Websocket messages."
)]
async fn echo(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    WebSocketUpgrade::new()
        .upgrade(req, res, |mut ws| async move {
            while let Some(msg) = ws.recv().await {
                let msg = if let Ok(msg) = msg {
                    msg
                } else {
                    // client disconnected
                    return;
                };
                tracing::info!("[ws] Received message: {:?}", msg);
                if ws.send(msg).await.is_err() {
                    // client disconnected
                    return;
                }
            }
        })
        .await
}

pub fn ws_router() -> Router {
    Router::new().push(Router::with_path("ws/echo").get(echo))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use salvo::conn::Acceptor;
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    #[tokio::test]
    async fn echoes_websocket_messages() {
        let acceptor = TcpListener::new("127.0.0.1:0").bind().await;
        let addr = acceptor.holdings()[0]
            .local_addr
            .clone()
            .into_std()
            .unwrap();
        tokio::spawn(Server::new(acceptor).serve(ws_router()));

        let (mut socket, _) = connect_async(format!("ws://{addr}/ws/echo")).await.unwrap();

        for _ in 0..10 {
            let message = Message::Text("hello".into());
            socket.send(message.clone()).await.unwrap();
            assert_eq!(socket.next().await.unwrap().unwrap(), message);
        }
    }
}
