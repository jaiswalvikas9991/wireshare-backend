use std::{collections::HashMap, sync::Arc};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use futures::{stream::SplitSink, SinkExt, StreamExt};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::time::{self, Duration};
use std::net::SocketAddr;


const ALPHABET_AND_NUMBERS: [char; 36] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];
struct State {
    users: HashMap<String, SplitSink<WebSocket, Message>>,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(State {
        users: HashMap::new(),
    }));
    let state_clone = state.clone();

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            let mut state = state_clone.lock().await;
            for (_, user) in state.users.iter_mut() {
                if user.send(Message::Text("heartbeat".to_string())).await.is_err() {
                    println!("Error while sending heartbeat");
                }
            }
        }
    });

    let router = Router::new()
        .route("/", get(websocket_handler))
        .route("/health", get(|| async { "ok" }))
        .layer(Extension(state));

    println!("Router initialized");


    let port: u16 = std::env::var("PORT")
    .unwrap_or_else(|_| "8080".into())
    .parse()
    .unwrap();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("Starting server on 0.0.0.0:{port}");
    axum_server::bind(addr)
    .serve(router.into_make_service())
    .await
    .unwrap();
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UserConnectedOutput<'a> {
    #[serde(rename = "type")]
    msg_type: u8,
    user_id: &'a str,
}
async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<Mutex<State>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Msg {
    to_user_id: String,
    msg: String,
}
async fn websocket(stream: WebSocket, state: Arc<Mutex<State>>) {
    // By splitting we can send and receive at the same time.
    let (sender, mut receiver) = stream.split();

    let user_id = nanoid!(10, &ALPHABET_AND_NUMBERS);
    state.lock().await.users.insert(user_id.clone(), sender);

    // This task will receive messages from this client.
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(json))) = receiver.next().await {
            let msg = serde_json::from_str::<Msg>(&json).unwrap();
            if let Some(user) = state_clone.lock().await.users.get_mut(&msg.to_user_id) {
                user.send(Message::Text(msg.msg)).await.unwrap();
            }
        }
    });

    let on_join_msg = {
        let msg = UserConnectedOutput {
            msg_type: 0,
            user_id: &user_id,
        };
        serde_json::to_string(&msg).unwrap()
    };
    if state
        .lock()
        .await
        .users
        .get_mut(&user_id)
        .unwrap()
        .send(Message::Text(on_join_msg))
        .await
        .is_err()
    {
        recv_task.abort();
    }

    tokio::select! {
            _ = (&mut recv_task) => {
                state.lock().await.users.remove(&user_id);
            }
    };
}
