use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, HeaderName, header},
    response::IntoResponse,
    routing::get,
};
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use tokio::select;
use wynnmap_types::{api::v2::RespWrapper, ws::TerrSockMessage};

use crate::{AnyError, etag::check_etag, header_date, state::TerritoryState};

pub(crate) fn router(state: Arc<TerritoryState>) -> axum::Router {
    axum::Router::new()
        .route("/list", get(terr_list))
        .route("/guilds", get(guild_list))
        .route("/guilds/ws", get(ws_handler))
        .with_state(state)
}

/// Common headers for responses from these endpoints
fn resp_headers(
    etag: &Arc<str>,
    expires: DateTime<Utc>,
    modified: DateTime<Utc>,
) -> [(HeaderName, String); 4] {
    [
        (
            header::CACHE_CONTROL,
            String::from("public, max-age=3600, must-revalidate"),
        ),
        (header::ETAG, format!("\"{etag}\"")),
        (header::EXPIRES, header_date(expires)),
        (header::LAST_MODIFIED, header_date(modified)),
    ]
}

#[tracing::instrument(skip(state, headers))]
async fn terr_list(
    State(state): State<Arc<TerritoryState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (territories, etag, modified, expires) = {
        let lock = state.inner.read().await;
        (
            lock.territories.clone(),
            lock.territories_etag.clone(),
            lock.territories_modified,
            lock.expires,
        )
    };

    let resp_headers = resp_headers(&etag, expires, modified);

    if check_etag(headers, &etag) {
        (StatusCode::NOT_MODIFIED, resp_headers, Body::empty()).into_response()
    } else {
        (resp_headers, Json(territories)).into_response()
    }
}

#[tracing::instrument(skip(state, headers))]
async fn guild_list(
    State(state): State<Arc<TerritoryState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (owners, etag, modified, expires) = {
        let lock = state.inner.read().await;
        (
            lock.owners.clone(),
            lock.owners_etag.clone(),
            lock.owners_modified,
            lock.expires,
        )
    };

    let resp_headers = resp_headers(&etag, expires, modified);

    if check_etag(headers, &etag) {
        (StatusCode::NOT_MODIFIED, resp_headers, Body::empty()).into_response()
    } else {
        (
            resp_headers,
            Json(RespWrapper {
                data: owners,
                updated: modified,
            }),
        )
            .into_response()
    }
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
                    let ts = { state.inner.read().await.last_updated };
                    socket.send(Message::Text(serde_json::to_string(&TerrSockMessage::LastUpdate { ts })?.into())).await?;
                }
            }
        }

        Ok(())
    }
}
