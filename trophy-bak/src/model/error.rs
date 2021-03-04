use actix_web::{dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
enum DataBaseError {
    NotFoundError { field: String },
    UpdateError { field: String },
    DeletionError { field: String },
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
            DataBaseError::UpdateError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            DataBaseError::DeletionError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Display, Error)]
enum EvaluationError {
    EarlyEvaluation { field: String },
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
            EvaluationError::EarlyEvaluation { .. } => StatusCode::from_u16(425).unwrap(),
        }
    }
}

#[derive(Debug, Display, Error)]
enum AuthenticationError {
    NoTokenError { field: String },
    AccessDeniedError { field: String },
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
