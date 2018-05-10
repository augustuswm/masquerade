use http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Result};
use actix_web::middleware::{Middleware, Response, Started};

use user::User;

#[derive(Debug)]
pub struct Admin;

impl<S> Middleware<S> for Admin {
    fn start(&self, req: &mut HttpRequest<S>) -> Result<Started> {
        if let Some(user) = req.extensions().get::<User>() {
            if user.is_admin() {
                Ok(Started::Done)
            } else {
                Ok(Started::Response(HttpResponse::new(
                    StatusCode::FORBIDDEN
                )))
            }
        } else {
            Ok(Started::Response(HttpResponse::new(
                StatusCode::UNAUTHORIZED
            )))
        }
    }

    fn response(&self, _: &mut HttpRequest<S>, resp: HttpResponse) -> Result<Response> {
        Ok(Response::Done(resp))
    }
}
