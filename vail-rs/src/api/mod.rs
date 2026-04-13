pub mod auth;
pub mod host;
pub mod sftp;

use axum::extract::State;
use axum::response::IntoResponse;
use sqlx::PgPool;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
}

impl<S> axum::extract::FromRef<S> for AppState
where
    S: Send + Sync,
{
    fn from_ref(state: &S) -> Self {
        State::from_ref(state)
    }
}
