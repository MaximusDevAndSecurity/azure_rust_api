use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse, Payload}, 
    Error, web, http::header::Authorization, HttpResponse,
};
use futures::future::{ok, Either, FutureExt, LocalBoxFuture};
use std::pin::Pin;
use std::task::{Context, Poll};
use crate::auth::validate_token;

pub struct JwtValidator;

impl<S, B> Transform<S> for JwtValidator
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtValidatorMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtValidatorMiddleware { service })
    }
}

pub struct JwtValidatorMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for JwtValidatorMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let token = req.headers().get(Authorization::bearer());

        match token {
            Some(token) if validate_token(token).is_ok() => {
                // Token is valid, continue processing the request
                let fut = self.service.call(req);
                fut.boxed_local()
            }
            _ => {
                // Token is missing or invalid
                let response = HttpResponse::Unauthorized().finish().into_body();
                let future = async { Ok(ServiceResponse::<B>::new(req.into_parts().0, response)) };
                future.boxed_local()
            }
        }
    }
}
