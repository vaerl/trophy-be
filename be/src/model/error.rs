use actix_web::{
    body::Body, dev::BaseHttpResponseBuilder, error, http::header, http::StatusCode,
    BaseHttpResponse, HttpResponse, HttpResponseBuilder,
};
use argon2::password_hash::HashError;
use thiserror::Error;
use xlsxwriter::XlsxError;

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
    CatchAllError { message: String },
    #[error("The resource already exists: {message}")]
    AlreadyExistsError { message: String },

    // eval-errors
    #[error("You tried to evaluate while teams are still playing: {message}")]
    EarlyEvaluationError { message: String },
    #[error("An error occurred while creating the excel-file: {message}")]
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
}

impl error::ResponseError for CustomError {
    fn error_response(&self) -> BaseHttpResponse<Body> {
        BaseHttpResponseBuilder::new(self.status_code())
            .append_header(("CONTENT_TYPE", "text/html; charset=utf-8"))
            .body(self.to_string())
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            // db-errors
            CustomError::NotFoundError { .. } => StatusCode::NOT_FOUND,
            CustomError::AlreadyExistsError { .. } => StatusCode::BAD_REQUEST,
            CustomError::CatchAllError { .. } => StatusCode::INTERNAL_SERVER_ERROR,

            // eval-errors
            // 425 -> too early, experimental API!
            CustomError::EarlyEvaluationError { .. } => StatusCode::from_u16(425).unwrap(),
            CustomError::XlsxError { .. } => StatusCode::INTERNAL_SERVER_ERROR,

            //auth-errors
            CustomError::NoTokenError { .. } => StatusCode::UNAUTHORIZED,
            CustomError::AccessDeniedError => StatusCode::FORBIDDEN,
            CustomError::BadPasswordError { .. } => StatusCode::BAD_REQUEST,
            CustomError::UnauthorizedError => StatusCode::UNAUTHORIZED,
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
            _ => CustomError::CatchAllError {
                message: err.to_string(),
            },
        }
    }
}

impl From<humantime::DurationError> for CustomError {
    fn from(err: humantime::DurationError) -> CustomError {
        CustomError::CatchAllError {
            message: err.to_string(),
        }
    }
}

impl From<std::num::ParseIntError> for CustomError {
    fn from(err: std::num::ParseIntError) -> CustomError {
        CustomError::CatchAllError {
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

impl From<jsonwebtoken::errors::Error> for CustomError {
    fn from(err: jsonwebtoken::errors::Error) -> CustomError {
        CustomError::NoTokenError {
            message: err.to_string(),
        }
    }
}

impl From<HashError> for CustomError {
    fn from(err: HashError) -> CustomError {
        CustomError::BadPasswordError {
            message: err.to_string(),
        }
    }
}
