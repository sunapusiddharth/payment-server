// src/ws/server.rs
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

type Tx = tokio::sync::mpsc::UnboundedSender<Message>;
type Rx = tokio::sync::mpsc::UnboundedReceiver<Message>;

pub struct WsServer {
    clients: Arc<Mutex<HashMap<String, Tx>>>, // user_id -> sender
}

impl WsServer {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn handle_connection(&self, user_id: String, ws_stream: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>) {
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        let (tx, mut rx): (Tx, Rx) = tokio::sync::mpsc::unbounded_channel();
        self.clients.lock().await.insert(user_id.clone(), tx);

        // Send welcome message
        let _ = ws_sender.send(Message::Text(r#"{"type":"welcome","message":"Connected"}"#.to_string())).await;

        // Handle incoming messages (ping/pong)
        let incoming = tokio::spawn(async move {
            while let Some(message) = ws_receiver.next().await {
                match message {
                    Ok(Message::Ping(data)) => {
                        let _ = ws_sender.send(Message::Pong(data)).await;
                    }
                    Ok(Message::Close(_)) => break,
                    _ => {}
                }
            }
        });

        // Handle outgoing messages
        let outgoing = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if ws_sender.send(message).await.is_err() {
                    break;
                }
            }
        });

        let _ = tokio::try_join!(incoming, outgoing);
        self.clients.lock().await.remove(&user_id);
    }

    pub async fn send_notification(&self, user_id: &str, message: &str) {
        if let Some(tx) = self.clients.lock().await.get(user_id) {
            let _ = tx.send(Message::Text(message.to_string()));
        }
    }
}