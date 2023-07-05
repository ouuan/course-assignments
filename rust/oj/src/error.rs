//! Custom errors.

use actix_web::{http::StatusCode, HttpRequest, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt::{Display, Formatter};

/// The type of the API error.
#[derive(Debug, Clone, Copy)]
pub enum ApiErrorType {
    InvalidArgument = 1,
    InvalidState = 2,
    NotFound = 3,
    RateLimit = 4,
    External = 5,
    Internal = 6,
}

impl ApiErrorType {
    fn reason(&self) -> &'static str {
        match self {
            ApiErrorType::InvalidArgument => "ERR_INVALID_ARGUMENT",
            ApiErrorType::InvalidState => "ERR_INVALID_STATE",
            ApiErrorType::NotFound => "ERR_NOT_FOUND",
            ApiErrorType::RateLimit => "ERR_RATE_LIMIT",
            ApiErrorType::External => "ERR_EXTERNAL",
            ApiErrorType::Internal => "ERR_INTERNAL",
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiErrorType::InvalidArgument => StatusCode::BAD_REQUEST,
            ApiErrorType::InvalidState => StatusCode::BAD_REQUEST,
            ApiErrorType::NotFound => StatusCode::NOT_FOUND,
            ApiErrorType::RateLimit => StatusCode::BAD_REQUEST,
            ApiErrorType::External => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorType::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// The custom API error type.
#[derive(Debug)]
pub struct ApiError {
    error_type: ApiErrorType,
    message: String,
}

impl ApiError {
    /// Construct an `ApiError`.
    pub fn new(error_type: ApiErrorType, message: String) -> Self {
        Self {
            error_type,
            message,
        }
    }

    /// Construct an `ApiError` with the not-found type.
    ///
    /// * `name`: the name of the thing not found, used in the error message.
    pub fn not_found(name: &str) -> Self {
        Self {
            error_type: ApiErrorType::NotFound,
            message: format!("{} not found.", name),
        }
    }

    /// Construct an `actix_web::Error` from another error representing an invalid-argument error.
    /// Mainly used as an `error_handler` of `actix_web::web::{JsonConfig, PathConfig}`, etc.
    pub fn invalid_argument(
        error: impl std::error::Error + Display,
        _: &HttpRequest,
    ) -> actix_web::Error {
        Self {
            error_type: ApiErrorType::InvalidArgument,
            message: error.to_string(),
        }
        .into()
    }
}

impl std::error::Error for ApiError {}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_type.reason())
    }
}

impl From<diesel::result::Error> for ApiError {
    fn from(error: diesel::result::Error) -> Self {
        ApiError::new(ApiErrorType::External, format!("database error: {}", error))
    }
}

impl From<r2d2::Error> for ApiError {
    fn from(error: r2d2::Error) -> Self {
        ApiError::new(
            ApiErrorType::External,
            format!("failed to get database connection: {}", error),
        )
    }
}

impl From<actix_web::error::BlockingError> for ApiError {
    fn from(_: actix_web::error::BlockingError) -> Self {
        ApiError::new(
            ApiErrorType::Internal,
            String::from("error running a blocking task on a thread pool"),
        )
    }
}

impl<T> From<async_channel::SendError<T>> for ApiError {
    fn from(error: async_channel::SendError<T>) -> Self {
        ApiError::new(
            ApiErrorType::Internal,
            format!("failed to add job to queue: {}", error),
        )
    }
}

impl From<std::io::Error> for ApiError {
    fn from(error: std::io::Error) -> Self {
        ApiError::new(ApiErrorType::External, format!("I/O error: {}", error))
    }
}

/// The JSON API response of an `ApiError`.
#[derive(Serialize)]
struct ErrorResponse {
    code: u8,
    reason: &'static str,
    message: String,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        if status_code == StatusCode::INTERNAL_SERVER_ERROR {
            log::error!("Internal server error: {}", self.message);
        }
        HttpResponse::build(status_code).json(ErrorResponse {
            code: self.error_type as u8,
            reason: self.error_type.reason(),
            message: self.message.clone(),
        })
    }

    fn status_code(&self) -> StatusCode {
        self.error_type.status_code()
    }
}

/// Type alias for `Result` of `ApiError`.
pub type ApiResult<T> = Result<T, ApiError>;
