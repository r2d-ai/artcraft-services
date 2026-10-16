use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::error::Error;
use actix_web::{web, App, HttpResponse};

use crate::http_server::endpoints::auth::auth_config_handler::auth_config_handler;
use crate::http_server::endpoints::auth::oidc_callback_handler::oidc_callback_handler;
use crate::http_server::endpoints::auth::oidc_login_handler::oidc_login_handler;

pub fn add_auth_routes<T, B>(app: App<T>) -> App<T>
where
  B: MessageBody,
  T: ServiceFactory<
    ServiceRequest,
    Config = (),
    Response = ServiceResponse<B>,
    Error = Error,
    InitError = (),
  >,
{
  app
    .service(
      web::resource("/v1/auth/config")
        .route(web::get().to(auth_config_handler))
        .route(web::head().to(|| HttpResponse::Ok())),
    )
    .service(
      web::resource("/v1/auth/oidc/login")
        .route(web::get().to(oidc_login_handler))
        .route(web::head().to(|| HttpResponse::Ok())),
    )
    .service(
      web::resource("/v1/auth/oidc/callback")
        .route(web::get().to(oidc_callback_handler))
        .route(web::head().to(|| HttpResponse::Ok())),
    )
}
