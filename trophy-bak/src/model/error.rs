use actix_web::{
    dev::HttpResponseBuilder, error, guard::Head, http::header, http::StatusCode, HttpResponse,
};
use derive_more::{Display, Error};
use xlsxwriter::XlsxError;

/// This enables me to simply call err.error_response() on errors, so all errors
/// have the correct status-codes.

#[derive(Debug, Display, Error)]
pub enum DataBaseError {
    NotFoundError { message: String },
    // Since sqlx does not differentiate between different database-errors and there are
    // not enough status-codes, I'm not differentiating here.
    CatchAllError { message: String },
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

#[derive(Debug, Display, Error)]
pub enum EvaluationError {
    EarlyEvaluationError { message: String },
    DataBaseError { message: String },
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

#[derive(Debug, Display, Error)]
pub enum AuthenticationError {
    NoTokenError { message: String },
    AccessDeniedError { message: String },
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
