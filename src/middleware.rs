use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse}, 
    Error, http, HttpResponse, body::BoxBody,
};
use futures::future::{self, Ready, FutureExt, LocalBoxFuture};
use std::task::{Context, Poll};
use crate::auth::validate_token;



pub struct JwtValidator;

impl<S> Transform<S, ServiceRequest> for JwtValidator
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = JwtValidatorMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ready(Ok(JwtValidatorMiddleware { service }))
    }
}

pub struct JwtValidatorMiddleware<S> {
    service: S,
}


impl<S> Service<ServiceRequest> for JwtValidatorMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let headers = req.headers();

        let auth_header = headers.get(http::header::AUTHORIZATION);

        if let Some(auth_header_value) = auth_header {
            if let Ok(auth_str) = auth_header_value.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    dotenv::dotenv().ok();
                    let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY must be set");
                    if validate_token(token, secret_key.as_bytes()).is_ok() {
                        // Token is valid, continue with the request
                        let fut = self.service.call(req);
                        return fut.boxed_local();
                    }
                }
            }
        }
        
        // Token is missing or invalid, respond with Unauthorized
        let response = HttpResponse::Unauthorized().finish().map_into_boxed_body();
        let service_response = ServiceResponse::new(req.into_parts().0, response);
        Box::pin(async move { Ok(service_response) })
    }
}
