use crate::{
    middleware::AuthInfo,
    model::{CustomError, History, LogLevel, SubjectType},
    ApiResult,
};
use actix::fut::{ready, Ready};
use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    http::Method,
    web::Data,
    Error, HttpMessage,
};
use futures::{future::LocalBoxFuture, FutureExt};
use sqlx::PgPool;
use std::{fmt::Display, rc::Rc};

pub struct LogMiddleware<S> {
    pool: Rc<Data<PgPool>>,
    service: Rc<S>,
}

pub struct LogMiddlewareFactory {
    pool: Rc<Data<PgPool>>,
}

impl LogMiddlewareFactory {
    pub fn new(pool: Data<PgPool>) -> Self {
        Self {
            pool: Rc::new(pool),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for LogMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = LogMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LogMiddleware {
            pool: Rc::clone(&self.pool),
            service: Rc::new(service),
        }))
    }
}

// largely taken from https://imfeld.dev/writing/actix-web-middleware
impl<S, B> Service<ServiceRequest> for LogMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;

    type Error = Error;

    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_service::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Clone the Rc pointers so we can move them into the async block.
        let srv = self.service.clone();
        let pool = self.pool.clone();
        async move {
            let auth = req.extensions().get::<AuthInfo>().cloned();
            // NOTE match_info works only after the request has been handled, see https://github.com/actix/actix-web/issues/1784
            let res = srv.call(req).await?;

            // fall back to path if pattern doesn't return a value
            let (path, subject_id) = match res.request().match_pattern() {
                Some(val) => (
                    val,
                    res.request()
                        .match_info()
                        .get("id")
                        .and_then(|s| s.parse::<i32>().ok()),
                ),
                None => (res.request().path().to_owned(), None),
            };

            match match_operation(res.request().method(), &path) {
                Err(err) => warn!("Could not extract operation-summary: {}", err),
                Ok(summary) => {
                    match auth {
                        Some(val) => {
                            let entry = History::create(
                                val.id,
                                summary.level.clone(),
                                summary.operation.clone(),
                                subject_id,
                                summary.subject_type.clone(),
                                &pool,
                            )
                            .await?;
                            log::log!(summary.level.into(), "{}", entry);
                        }
                        None => info!("Executed '{}' without authentication.", summary),
                    };
                }
            }
            Ok(res)
        }
        .boxed_local()
    }
}

struct OperationSummary {
    operation: String,
    /// Because we rely on this value in the frontend, these values should be normed (i.e. we require an enum here).
    subject_type: SubjectType,
    level: LogLevel,
}

/// This implements some shorthands for easier usage.
/// Groups by both [SubjectType] or [Operation], depending on similarities.
///
/// NOTE I don't yet know if these lines are worth the (hopefully) improved reading-
/// experience in [match_operation].
impl OperationSummary {
    fn eval(operation: String, level: LogLevel) -> Self {
        OperationSummary {
            operation,
            subject_type: SubjectType::Eval,
            level,
        }
    }

    fn create(subject_type: SubjectType) -> Self {
        OperationSummary {
            operation: "create".to_string(),
            subject_type,
            level: LogLevel::Info,
        }
    }

    fn get(subject_type: SubjectType) -> Self {
        OperationSummary {
            operation: "get".to_string(),
            subject_type,
            level: LogLevel::Debug,
        }
    }

    fn update(subject_type: SubjectType) -> Self {
        OperationSummary {
            operation: "update".to_string(),
            subject_type,
            level: LogLevel::Debug,
        }
    }

    fn delete(subject_type: SubjectType) -> Self {
        OperationSummary {
            operation: "delete".to_string(),
            subject_type,
            level: LogLevel::Info,
        }
    }

    fn get_all(subject_type: SubjectType) -> Self {
        OperationSummary {
            operation: "get all".to_string(),
            subject_type,
            level: LogLevel::Debug,
        }
    }

    fn pending(subject_type: SubjectType) -> Self {
        OperationSummary {
            operation: "get pending".to_string(),
            subject_type,
            level: LogLevel::Debug,
        }
    }

    fn finished(subject_type: SubjectType) -> Self {
        OperationSummary {
            operation: "get finished".to_string(),
            subject_type,
            level: LogLevel::Debug,
        }
    }

    fn import(subject_type: SubjectType) -> Self {
        OperationSummary {
            operation: "import".to_string(),
            subject_type,
            level: LogLevel::Debug,
        }
    }
}

impl Display for OperationSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OperationSummary(operation: {}, subject: {}, level: {})",
            self.operation, self.subject_type, self.level
        )
    }
}

fn match_operation(method: &Method, path: &str) -> ApiResult<OperationSummary> {
    match path {
        "/eval" => Ok(OperationSummary::eval(
            "evaluate trophy".to_string(),
            LogLevel::Warn,
        )),
        "/eval/sheet" => Ok(OperationSummary::eval(
            "download sheet".to_string(),
            LogLevel::Debug,
        )),
        "/eval/done" => Ok(OperationSummary::eval(
            "check if evaluation is done".to_string(),
            LogLevel::Debug,
        )),
        "/games" => match *method {
            Method::GET => Ok(OperationSummary::get_all(SubjectType::Game)),
            Method::POST => Ok(OperationSummary::create(SubjectType::Game)),
            _ => Err(CustomError::UnsupportedMethod {
                method: method.to_string(),
                path: path.to_string(),
            }),
        },
        "/games/pending/amount" => Ok(OperationSummary::pending(SubjectType::Game)),
        "/games/finished" => Ok(OperationSummary::finished(SubjectType::Game)),
        "/games/{id}" => match *method {
            Method::GET => Ok(OperationSummary::get(SubjectType::Game)),
            Method::PUT => Ok(OperationSummary::update(SubjectType::Game)),
            Method::DELETE => Ok(OperationSummary::delete(SubjectType::Game)),
            _ => Err(CustomError::UnsupportedMethod {
                method: method.to_string(),
                path: path.to_string(),
            }),
        },
        "/games/{id}/pending/amount" => Ok(OperationSummary {
            operation: "get the amount of pending teams for game".to_string(),
            subject_type: SubjectType::Game,
            level: LogLevel::Debug,
        }),
        "/history" => Ok(OperationSummary::get_all(SubjectType::History)),
        "/import" => Ok(OperationSummary::import(SubjectType::Team)),
        "/ping" => Ok(OperationSummary {
            operation: "ping".to_string(),
            subject_type: SubjectType::General,
            level: LogLevel::Debug,
        }),
        "/done" => Ok(OperationSummary {
            operation: "check if trophy is done".to_string(),
            subject_type: SubjectType::General,
            level: LogLevel::Debug,
        }),
        "/outcomes" => match *method {
            Method::GET => Ok(OperationSummary::get_all(SubjectType::Outcome)),
            Method::PUT => Ok(OperationSummary::update(SubjectType::Outcome)),
            _ => Err(CustomError::UnsupportedMethod {
                method: method.to_string(),
                path: path.to_string(),
            }),
        },
        "/outcomes/games/{id}" => Ok(OperationSummary::get_all(SubjectType::Outcome)),
        "/outcomes/teams/{id}" => Ok(OperationSummary::get_all(SubjectType::Outcome)),
        "/teams" => match *method {
            Method::GET => Ok(OperationSummary::get_all(SubjectType::Team)),
            Method::POST => Ok(OperationSummary::create(SubjectType::Team)),
            _ => Err(CustomError::UnsupportedMethod {
                method: method.to_string(),
                path: path.to_string(),
            }),
        },
        "/teams/pending/amount" => Ok(OperationSummary::pending(SubjectType::Team)),
        "/teams/{id}" => match *method {
            Method::GET => Ok(OperationSummary::get(SubjectType::Team)),
            Method::PUT => Ok(OperationSummary::update(SubjectType::Team)),
            Method::DELETE => Ok(OperationSummary::delete(SubjectType::Team)),
            _ => Err(CustomError::UnsupportedMethod {
                method: method.to_string(),
                path: path.to_string(),
            }),
        },
        "/user/status" => Ok(OperationSummary {
            operation: "check if user is logged in".to_string(),
            subject_type: SubjectType::General,
            level: LogLevel::Debug,
        }),
        "/users" => match *method {
            Method::GET => Ok(OperationSummary::get_all(SubjectType::User)),
            Method::POST => Ok(OperationSummary::create(SubjectType::User)),
            _ => Err(CustomError::UnsupportedMethod {
                method: method.to_string(),
                path: path.to_string(),
            }),
        },
        "/users/{id}" => match *method {
            Method::GET => Ok(OperationSummary::get(SubjectType::User)),
            Method::PUT => Ok(OperationSummary::update(SubjectType::User)),
            Method::DELETE => Ok(OperationSummary::delete(SubjectType::User)),
            _ => Err(CustomError::UnsupportedMethod {
                method: method.to_string(),
                path: path.to_string(),
            }),
        },
        "/users/{id}/game" => Ok(OperationSummary::get(SubjectType::Game)),
        "/login" => Ok(OperationSummary {
            operation: "login".to_string(),
            subject_type: SubjectType::User,
            level: LogLevel::Debug,
        }),
        "/logout" => Ok(OperationSummary {
            operation: "logout".to_string(),
            subject_type: SubjectType::User,
            level: LogLevel::Debug,
        }),
        _ => Err(CustomError::UnsupportedPath {
            path: path.to_string(),
        }),
    }
}
