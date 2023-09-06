use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use log::error;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct LocalServer {
    router: Router,
    shutdown_rx: Receiver<()>,
    listener: TcpListener,
}

impl<'a> LocalServer {
    pub fn new() -> Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::channel::<()>(1);
        // Port number of 0 requests OS to find an available port.
        let listener = TcpListener::bind("0.0.0.0:0")?;

        let shared_state = Arc::new(AppState::new(tx));
        let router = Router::new()
            .route("/callback", post(Self::callback))
            .with_state(shared_state);

        Ok(Self {
            router,
            shutdown_rx: rx,
            listener,
        })
    }

    pub fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        self.listener.local_addr()
    }

    pub async fn start(mut self) -> Result<()> {
        axum::Server::from_tcp(self.listener)?
            .serve(self.router.into_make_service())
            .with_graceful_shutdown(async {
                let _ = &self.shutdown_rx.recv().await;
            })
            .await?;

        Ok(())
    }

    async fn callback(
        State(state): State<Arc<AppState>>,
        Query(params): Query<CallbackParams>,
    ) -> Result<Json<Value>, AppError> {
        state.shutdown().await?;

        match params.code {
            Some(code) => {
                println!("auth_code: {}", code);

                // TODO: 1. Get access token using the authorization code

                // TODO: 2. Store the access token locally

                Ok(Json(json!({ "success": true })))
            }
            None => {
                error!("User denied consent. Try again.");

                Ok(Json(json!({ "success": true })))
            }
        }
    }
}

#[derive(Deserialize)]
struct CallbackParams {
    code: Option<String>,
}

#[derive(Clone)]
struct AppState {
    shutdown_tx: Sender<()>,
}

impl AppState {
    fn new(shutdown_tx: Sender<()>) -> Self {
        Self { shutdown_tx }
    }

    async fn shutdown(&self) -> Result<()> {
        self.shutdown_tx.send(()).await?;

        Ok(())
    }
}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
