use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("authentication required")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("invalid request: {0}")]
    BadRequest(String),
    #[error("rate limit exceeded")]
    RateLimited,
    #[error("database unavailable")]
    Database(#[from] sqlx::Error),
    #[error("internal error")]
    Internal,
}

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorDetail,
}
#[derive(Serialize)]
struct ErrorDetail {
    code: &'static str,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, code) = match self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
            Self::Forbidden => (StatusCode::FORBIDDEN, "forbidden"),
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            Self::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
            Self::Database(_) | Self::Internal => {
                tracing::error!(error = %self, "request failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            }
        };
        (
            status,
            Json(ErrorBody {
                error: ErrorDetail {
                    code,
                    message: self.to_string(),
                },
            }),
        )
            .into_response()
    }
}
