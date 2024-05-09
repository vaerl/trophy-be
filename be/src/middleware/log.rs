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
use std::rc::Rc;

use crate::{
    middleware::AuthInfo,
    model::{History, LogLevel},
};

pub struct LogMiddleware<S> {
    pool: Rc<Data<PgPool>>,
    service: Rc<S>,
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
            // fall back to path if pattern doesn't return a value
            let path = match req.match_pattern() {
                Some(val) => val,
                None => req.path().to_owned(),
            };
            let (action, level) = match_operation(req.method(), &path);

            match auth {
                Some(val) => {
                    History::create(val.id, level.clone(), action.clone(), &pool).await?;
                    log::log!(level.into(), "{}", action);
                }
                None => info!("Executed '{}' without authentication.", action),
            };
            let res = srv.call(req).await?;
            Ok(res)
        }
        .boxed_local()
    }
}

fn match_operation(method: &Method, path: &str) -> (String, LogLevel) {
    match path {
        "/eval" => (format!("evaluate trophy"), LogLevel::Important),
        "/eval/sheet" => (format!("download sheet"), LogLevel::Debug),
        "/eval/done" => (format!("check if trophy is evaluated"), LogLevel::Debug),
        _ => todo!(),
    }
}

pub struct LogMiddlewareFactory {
    pool: Rc<Data<PgPool>>,
}

impl LogMiddlewareFactory {
    pub fn new(pool: Data<PgPool>) -> Self {
        LogMiddlewareFactory {
            pool: Rc::new(pool),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for LogMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = LogMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LogMiddleware {
            pool: self.pool.clone(),
            service: Rc::new(service),
        }))
    }
}
