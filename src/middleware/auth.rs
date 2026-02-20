use std::rc::Rc;

use actix::fut::{Ready, ready};
use actix_service::{Service, Transform};
use actix_web::{
    Error, FromRequest, HttpMessage,
    dev::{ServiceRequest, ServiceResponse},
    web::Data,
};
use futures::{FutureExt, future::LocalBoxFuture};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    ApiResult,
    model::{CustomError, UserRole, UserToken},
};

#[derive(Debug)]
pub struct Auth {
    pub id: Uuid,
    pub role: UserRole,
}

pub type AuthInfo = Rc<Auth>;

pub struct Authenticated(AuthInfo);

impl Authenticated {
    pub fn has_roles(&self, roles: Vec<UserRole>) -> ApiResult<&Self> {
        if roles.contains(&self.role) {
            Ok(self)
        } else {
            Err(CustomError::AccessDeniedError)
        }
    }
}

impl FromRequest for Authenticated {
    type Error = CustomError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<AuthInfo>().cloned();
        let result = match value {
            Some(v) => Ok(Authenticated(v)),
            None => Err(CustomError::UnauthorizedError),
        };
        ready(result)
    }
}

impl std::ops::Deref for Authenticated {
    type Target = AuthInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct AuthMiddleware<S> {
    pool: Rc<Data<PgPool>>,
    service: Rc<S>,
}

// largely taken from https://imfeld.dev/writing/actix-web-middleware
impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
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
            let user = UserToken::try_into_user(&req, &pool).await;

            // try to find a user, later code can decide if an authentication is necessary
            if let Ok(user) = user {
                let auth = Auth {
                    id: user.id,
                    role: user.role,
                };
                req.extensions_mut().insert::<AuthInfo>(Rc::new(auth));
            }

            let res = srv.call(req).await?;

            Ok(res)
        }
        .boxed_local()
    }
}

pub struct AuthMiddlewareFactory {
    pool: Rc<Data<PgPool>>,
}

impl AuthMiddlewareFactory {
    pub fn new(pool: Data<PgPool>) -> Self {
        AuthMiddlewareFactory {
            pool: Rc::new(pool),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            pool: self.pool.clone(),
            service: Rc::new(service),
        }))
    }
}
