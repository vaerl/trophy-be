use actix_web::{dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse};
use argon2::password_hash::HashError;
use thiserror::Error;
use xlsxwriter::XlsxError;

/// This enables me to simply call err.error_response() on errors, so all errors
/// have the correct status-codes.

#[derive(Debug, Error)]
pub enum DataBaseError {
    #[error("The requested resource could not be found: {message}")]
    NotFoundError { message: String },
    // Since sqlx does not differentiate between different database-errors and there are
    // not enough status-codes, I'm not differentiating here.
    #[error("An error occurred while interacting with the database: {message}")]
    CatchAllError { message: String },
    #[error("The resource already exists: {message}")]
    AlreadyExistsError { message: String },
}

impl error::ResponseError for DataBaseError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            DataBaseError::NotFoundError { .. } => StatusCode::NOT_FOUND,
            DataBaseError::AlreadyExistsError { .. } => StatusCode::BAD_REQUEST,
            DataBaseError::CatchAllError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<sqlx::Error> for DataBaseError {
    fn from(err: sqlx::Error) -> DataBaseError {
        match err {
            sqlx::Error::RowNotFound | sqlx::Error::ColumnNotFound(_) => {
                DataBaseError::NotFoundError {
                    message: err.to_string(),
                }
            }
            _ => DataBaseError::CatchAllError {
                message: err.to_string(),
            },
        }
    }
}

impl From<humantime::DurationError> for DataBaseError {
    fn from(err: humantime::DurationError) -> DataBaseError {
        DataBaseError::CatchAllError {
            message: err.to_string(),
        }
    }
}

impl From<std::num::ParseIntError> for DataBaseError {
    fn from(err: std::num::ParseIntError) -> DataBaseError {
        DataBaseError::CatchAllError {
            message: err.to_string(),
        }
    }
}

#[derive(Debug, Error)]
pub enum EvaluationError {
    #[error("You tried to evaluate while teams are still playing: {message}")]
    EarlyEvaluationError { message: String },
    #[error("A database-error occurred: {message}")]
    DataBaseError { message: String },
    #[error("An error occurred while creating the excel-file: {message}")]
    XlsxError { message: String },
}

impl error::ResponseError for EvaluationError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            // 425 -> too early, experimental API!
            EvaluationError::EarlyEvaluationError { .. } => StatusCode::from_u16(425).unwrap(),
            EvaluationError::DataBaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            EvaluationError::XlsxError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<DataBaseError> for EvaluationError {
    fn from(err: DataBaseError) -> EvaluationError {
        EvaluationError::DataBaseError {
            message: err.to_string(),
        }
    }
}

impl From<XlsxError> for EvaluationError {
    fn from(err: XlsxError) -> EvaluationError {
        EvaluationError::XlsxError {
            message: err.to_string(),
        }
    }
}

impl From<std::io::Error> for EvaluationError {
    fn from(err: std::io::Error) -> EvaluationError {
        EvaluationError::XlsxError {
            message: err.to_string(),
        }
    }
}

#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("The request did not contain an access-token: {message}")]
    NoTokenError { message: String },
    #[error("You are not allowed to access this resource")]
    AccessDeniedError { message: String },
    #[error("A database-error occurred: {message}")]
    DataBaseError { message: String },
    #[error("The provided password was empty or wrong: {message}")]
    BadPasswordError { message: String },
    #[error("You must log in first!")]
    UnauthorizedError,
}

impl error::ResponseError for AuthenticationError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            AuthenticationError::NoTokenError { .. } => StatusCode::UNAUTHORIZED,
            AuthenticationError::AccessDeniedError { .. } => StatusCode::FORBIDDEN,
            AuthenticationError::DataBaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AuthenticationError::BadPasswordError { .. } => StatusCode::BAD_REQUEST,
            AuthenticationError::UnauthorizedError => StatusCode::UNAUTHORIZED,
        }
    }
}

impl From<actix_web::http::header::ToStrError> for AuthenticationError {
    fn from(err: actix_web::http::header::ToStrError) -> AuthenticationError {
        AuthenticationError::NoTokenError {
            message: err.to_string(),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AuthenticationError {
    fn from(err: jsonwebtoken::errors::Error) -> AuthenticationError {
        AuthenticationError::NoTokenError {
            message: err.to_string(),
        }
    }
}

impl From<DataBaseError> for AuthenticationError {
    fn from(err: DataBaseError) -> AuthenticationError {
        AuthenticationError::DataBaseError {
            message: err.to_string(),
        }
    }
}

impl From<HashError> for AuthenticationError {
    fn from(err: HashError) -> AuthenticationError {
        AuthenticationError::BadPasswordError {
            message: err.to_string(),
        }
    }
}
