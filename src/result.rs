use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
}

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!(
            "Fell out of request handler with error: {} {:?}",
            self.0,
            self.0
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: format!("{} {:?}", self.0, self.0),
            }),
        )
            .into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
