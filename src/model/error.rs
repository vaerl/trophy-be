use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse, HttpResponseBuilder,
};
use argon2::password_hash;
use serde::Serialize;
use thiserror::Error;
use xlsxwriter::XlsxError;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// TODO check if there are unused variants

/// This enables me to simply call err.error_response() on errors, so all errors
/// have the correct status-codes.

#[derive(Debug, Error)]
pub enum CustomError {
    // db-errors
    #[error("The requested resource could not be found: {message}")]
    NotFoundError { message: String },
    // Since sqlx does not differentiate between different database-errors and there are
    // not enough status-codes, I'm not differentiating here.
    #[error("An error occurred while interacting with the database: {message}")]
    DatabaseError { message: String },
    #[error("An error occurred while trying to parse something: {message}")]
    ParseError { message: String },
    #[error("An error occurred in/with HumanTime : {message}")]
    HumanTimeError { message: String },
    #[error("An error occurred in/with Actix: {message}")]
    ActixError { message: String },
    #[error("The resource already exists: {message}")]
    AlreadyExistsError { message: String },
    #[error("No data when there should have been some: {message}")]
    NoDataSentError { message: String },

    // eval-errors
    #[error("You tried to evaluate while teams are still playing: {message}")]
    EarlyEvaluationError { message: String },
    #[error("An error occurred while writing an excel-file: {message}")]
    XlsxError { message: String },

    // auth-errors
    #[error("The request did not contain an access-token: {message}")]
    NoTokenError { message: String },
    #[error("You are not allowed to access this resource!")]
    AccessDeniedError,
    #[error("The provided password was empty or wrong: {message}")]
    BadPasswordError { message: String },
    #[error("You must log in first!")]
    UnauthorizedError,

    // websocket-errors
    #[error("Could not send the websocket-message: {message}")]
    SendError { message: String },

    // log-errors
    #[error("Unsupported path: {path}")]
    UnsupportedPath { path: String },
    #[error("Unsupported method '{method}' for path '{path}'.")]
    UnsupportedMethod { method: String, path: String },

    // import-errors
    #[error("An error occurred while reading an excel-file: {message}")]
    CalmineError { message: String },
}

impl error::ResponseError for CustomError {
    fn error_response(&self) -> HttpResponse {
        let response = ErrorResponse {
            error: self.to_string(),
        };

        match serde_json::to_string(&response) {
            Ok(json) => HttpResponseBuilder::new(self.status_code())
                .content_type(ContentType::json())
                .body(json),
            Err(_) => HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR)
                .content_type(ContentType::json())
                .body(format!(
                    r#"{{error: "Error while serializing actual error: {}"}}"#,
                    self
                )),
        }
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            // db-errors
            CustomError::NotFoundError { .. } => StatusCode::NOT_FOUND,
            CustomError::AlreadyExistsError { .. } => StatusCode::BAD_REQUEST,
            CustomError::DatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::ParseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::ActixError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::HumanTimeError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::NoDataSentError { .. } => StatusCode::BAD_REQUEST,

            // eval-errors
            // 425 -> too early, experimental API!
            CustomError::EarlyEvaluationError { .. } => StatusCode::from_u16(425).unwrap(),
            CustomError::XlsxError { .. } => StatusCode::INTERNAL_SERVER_ERROR,

            //auth-errors
            CustomError::NoTokenError { .. } => StatusCode::UNAUTHORIZED,
            CustomError::AccessDeniedError => StatusCode::FORBIDDEN,
            CustomError::BadPasswordError { .. } => StatusCode::BAD_REQUEST,
            CustomError::UnauthorizedError => StatusCode::UNAUTHORIZED,

            // websocket-errors
            CustomError::SendError { .. } => StatusCode::INTERNAL_SERVER_ERROR,

            // logs
            CustomError::UnsupportedPath { .. } => StatusCode::BAD_REQUEST,
            CustomError::UnsupportedMethod { .. } => StatusCode::BAD_REQUEST,

            // import
            CustomError::CalmineError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<sqlx::Error> for CustomError {
    fn from(err: sqlx::Error) -> CustomError {
        match err {
            sqlx::Error::RowNotFound | sqlx::Error::ColumnNotFound(_) => {
                CustomError::NotFoundError {
                    message: err.to_string(),
                }
            }
            _ => CustomError::DatabaseError {
                message: err.to_string(),
            },
        }
    }
}

impl From<humantime::DurationError> for CustomError {
    fn from(err: humantime::DurationError) -> CustomError {
        CustomError::HumanTimeError {
            message: err.to_string(),
        }
    }
}

impl From<std::num::ParseIntError> for CustomError {
    fn from(err: std::num::ParseIntError) -> CustomError {
        CustomError::ParseError {
            message: err.to_string(),
        }
    }
}

impl From<XlsxError> for CustomError {
    fn from(err: XlsxError) -> CustomError {
        CustomError::XlsxError {
            message: err.to_string(),
        }
    }
}

impl From<std::io::Error> for CustomError {
    fn from(err: std::io::Error) -> CustomError {
        CustomError::XlsxError {
            message: err.to_string(),
        }
    }
}

impl From<actix_web::http::header::ToStrError> for CustomError {
    fn from(err: actix_web::http::header::ToStrError) -> CustomError {
        CustomError::NoTokenError {
            message: err.to_string(),
        }
    }
}

impl From<actix_web::error::Error> for CustomError {
    fn from(err: actix_web::error::Error) -> CustomError {
        CustomError::ActixError {
            message: err.to_string(),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for CustomError {
    fn from(err: jsonwebtoken::errors::Error) -> CustomError {
        CustomError::NoTokenError {
            message: err.to_string(),
        }
    }
}

impl From<password_hash::Error> for CustomError {
    fn from(err: password_hash::Error) -> CustomError {
        CustomError::BadPasswordError {
            message: err.to_string(),
        }
    }
}

impl From<calamine::XlsxError> for CustomError {
    fn from(err: calamine::XlsxError) -> CustomError {
        CustomError::BadPasswordError {
            message: err.to_string(),
        }
    }
}

impl From<calamine::DeError> for CustomError {
    fn from(err: calamine::DeError) -> CustomError {
        CustomError::BadPasswordError {
            message: err.to_string(),
        }
    }
}
