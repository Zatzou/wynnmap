use std::sync::Arc;

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
use wynnmap_types::{api::v2::RespWrapper, ws::TerrSockMessage};

use crate::{AnyError, state::TerritoryState};

pub(crate) fn router(state: Arc<TerritoryState>) -> axum::Router {
    axum::Router::new()
        .route("/list", get(terr_list))
        .route("/guilds", get(guild_list))
        .route("/guilds/ws", get(ws_handler))
        .with_state(state)
}

#[tracing::instrument(skip(state))]
async fn terr_list(State(state): State<Arc<TerritoryState>>) -> impl IntoResponse {
    let (territories, updated, expires) = {
        let lock = state.inner.read().await;
        (lock.territories.clone(), lock.last_updated, lock.expires)
    };

    (
        [
            (header::CACHE_CONTROL, String::from("public, max-age=10")),
            (
                header::EXPIRES,
                expires.map(|d| d.to_rfc2822()).unwrap_or_default(),
            ),
            (
                header::LAST_MODIFIED,
                updated.map(|d| d.to_rfc2822()).unwrap_or_default(),
            ),
        ],
        Json(territories.clone()),
    )
}

#[tracing::instrument(skip(state))]
async fn guild_list(State(state): State<Arc<TerritoryState>>) -> impl IntoResponse {
    let (owners, updated, expires) = {
        let lock = state.inner.read().await;
        (lock.owners.clone(), lock.last_updated, lock.expires)
    };

    (
        [
            (header::CACHE_CONTROL, String::from("public, max-age=10")),
            (
                header::EXPIRES,
                expires.map(|d| d.to_rfc2822()).unwrap_or_default(),
            ),
            (
                header::LAST_MODIFIED,
                updated.map(|d| d.to_rfc2822()).unwrap_or_default(),
            ),
        ],
        Json(RespWrapper {
            data: owners,
            updated: updated.unwrap_or_default(),
        }),
    )
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<TerritoryState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|s| async {
        handle_socket(s, state).await;
    })
}

async fn handle_socket(socket: WebSocket, state: Arc<TerritoryState>) {
    state.active_conn.add(1, &[]);
    let bc_recv = state.bc_recv.resubscribe();

    if let Err(e) = handle_socket_inner(socket, bc_recv, state.clone()).await {
        tracing::error!("Error handling socket: {:?}", e);
    }

    state.active_conn.add(-1, &[]);

    async fn handle_socket_inner(
        mut socket: WebSocket,
        mut bc_recv: tokio::sync::broadcast::Receiver<TerrSockMessage>,
        state: Arc<TerritoryState>,
    ) -> Result<(), AnyError> {
        loop {
            select! {
                // respond to received pings and close messages
                s = socket.recv() => {
                    if let Some(Ok(msg)) = s {
                        match msg {
                            Message::Ping(data) => {
                                socket.send(Message::Pong(data)).await?;
                            }
                            Message::Close(frame) => {
                                socket.send(Message::Close(frame)).await?;
                                break;
                            }
                            _ => {}
                        }
                    } else {
                        break;
                    }
                }
                // send messages from the broadcast channel
                m = bc_recv.recv() => {
                    if let Ok(msg) = m {
                        socket
                            .send(Message::Text(
                                serde_json::to_string(&msg)?.into(),
                            ))
                            .await?;
                    }
                }
                // send the last updated timestamp every 30 seconds
                _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
                    let ts = { state.inner.read().await.last_updated }.unwrap_or_default();
                    socket.send(Message::Text(serde_json::to_string(&TerrSockMessage::LastUpdate { ts })?.into())).await?;
                }
            }
        }

        Ok(())
    }
}
