use std::{sync::Arc, time::Duration};

use axum::{
    Json,
    body::Body,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, header},
    response::IntoResponse,
    routing::get,
};
use reqwest::StatusCode;
use tokio::{select, sync::broadcast, time::timeout};
use wynnmap_types::terr::MapState;

use crate::{AnyError, etag::check_etag, header_date, state::TerritoryState};

pub fn router(state: Arc<TerritoryState>) -> axum::Router {
    axum::Router::new()
        .route("/list", get(terr_list))
        .route("/state", get(map_state))
        .route("/state/ws", get(ws_handler))
        .with_state(state)
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

    let resp_headers = [
        (
            header::CACHE_CONTROL,
            String::from("public, max-age=10, immutable, must-revalidate"),
        ),
        (header::ETAG, format!("\"{etag}\"")),
        (header::EXPIRES, header_date(expires)),
        (header::LAST_MODIFIED, header_date(modified)),
    ];

    if check_etag(&headers, &etag) {
        (StatusCode::NOT_MODIFIED, resp_headers, Body::empty()).into_response()
    } else {
        (resp_headers, Json(territories)).into_response()
    }
}

#[tracing::instrument(skip(state))]
async fn map_state(State(state): State<Arc<TerritoryState>>) -> impl IntoResponse {
    let (state, expires, timestamps) = {
        let lock = state.inner.read().await;
        (lock.state.clone(), lock.expires, lock.timestamps)
    };

    let resp_headers = [
        (
            header::CACHE_CONTROL,
            String::from("public, max-age=10, immutable, must-revalidate"),
        ),
        (header::EXPIRES, header_date(expires)),
        (
            header::LAST_MODIFIED,
            header_date(timestamps.changed.unwrap_or_default()),
        ),
    ];

    (
        resp_headers,
        Json(MapState {
            terrs: state,
            timestamps,
        }),
    )
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<TerritoryState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |s| async move {
        state.ws_conns.add(1, &[]);
        let bc_recv = state.bc_bytes.resubscribe();

        if let Err(e) = handle_socket(s, bc_recv).await {
            tracing::error!("Error handling socket: {:?}", e);
        }

        state.ws_conns.add(-1, &[]);
    })
}

async fn handle_socket(
    mut socket: WebSocket,
    mut bc_recv: broadcast::Receiver<Arc<Vec<u8>>>,
) -> Result<(), AnyError> {
    loop {
        select! {
            // respond to received pings and close messages
            s = socket.recv() => {
                if let Some(Ok(msg)) = s {
                    match msg {
                        Message::Ping(data) => {
                            if data.len() > 32 { break; }
                            socket.send(Message::Pong(data)).await?;
                        }
                        Message::Close(frame) => {
                            let _ = timeout(Duration::from_secs(5), socket.send(Message::Close(frame))).await;
                            break;
                        }
                        _ => { break; }
                    }
                } else {
                    break;
                }
            }

            // send messages from the broadcast channel
            Ok(msg) = bc_recv.recv() => {
                socket
                    .send(Message::Binary((*msg).clone().into()))
                    .await?;
            }
        }
    }

    Ok(())
}
