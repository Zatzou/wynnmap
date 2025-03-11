use axum::{
    Json,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::header,
    response::IntoResponse,
    routing::get,
};
use tokio::select;
use wynnmap_types::ws::TerrSockMessage;

use crate::state::TerritoryState;

pub(crate) fn router(state: TerritoryState) -> axum::Router {
    axum::Router::new()
        .route("/list", get(terr_list))
        .route("/extra", get(extra_data))
        .route("/ws", get(ws_handler))
        .with_state(state)
}

async fn terr_list(State(state): State<TerritoryState>) -> impl IntoResponse {
    let read = state.inner.read().await;

    let age = (read.expires - chrono::Utc::now()).num_seconds();

    (
        [
            (header::CACHE_CONTROL, String::from("public, max-age=10")),
            (header::AGE, age.to_string()),
            (header::EXPIRES, read.expires.to_rfc2822()),
        ],
        Json(read.territories.clone()),
    )
}

async fn extra_data(State(state): State<TerritoryState>) -> impl IntoResponse {
    let read = state.extra.read().await;

    (
        [(header::CACHE_CONTROL, "public, max-age=3600")],
        Json(read.clone()),
    )
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<TerritoryState>,
) -> impl IntoResponse {
    ws.on_upgrade(|s| async {
        handle_socket(s, state).await;
    })
}

async fn handle_socket(mut socket: WebSocket, state: TerritoryState) {
    let mut bc_recv = state.bc_recv.resubscribe();

    socket
        .send(Message::Text(
            serde_json::to_string(&TerrSockMessage::Territory(
                state.inner.read().await.territories.clone(),
            ))
            .unwrap()
            .into(),
        ))
        .await
        .unwrap();

    loop {
        select! {
            s = socket.recv() => {
                if let Some(Ok(msg)) = s {
                    match msg {
                        Message::Ping(data) => {
                            socket.send(Message::Pong(data)).await.unwrap();
                        }
                        Message::Close(frame) => {
                            socket.send(Message::Close(frame)).await.unwrap();
                            break;
                        }
                        _ => {}
                    }
                } else {
                    break;
                }
            }
            m = bc_recv.recv() => {
                if let Ok(msg) = m {
                    socket
                        .send(Message::Text(
                            serde_json::to_string(&msg).unwrap().into(),
                        ))
                        .await
                        .unwrap();
                }
            }
        }
    }
}
