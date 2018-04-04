use http::{header, StatusCode};
use actix_web::{HttpRequest, HttpResponse, Result};
use actix_web::middleware::{Middleware, Response, Started};

use api::State;
use user::User;

#[derive(Debug)]
pub struct Auth;

impl Middleware<State> for Auth {
    fn start(&self, req: &mut HttpRequest<State>) -> Result<Started> {
        if let Some(auth) = req.clone().headers_mut().get(header::AUTHORIZATION) {
            if let Ok(Some(user)) = req.state()
                .users()
                .get(&"users".to_string(), auth.to_str().unwrap())
            {
                req.extensions().insert(user);
                Ok(Started::Done)
            } else {
                Ok(Started::Response(HttpResponse::new(
                    StatusCode::FORBIDDEN,
                    "".into(),
                )))
            }
        // req.extensions().insert(User::new(
        //     "user-id".into(),
        //     "key".into(),
        //     "secret".into(),
        //     true,
        // ));
        } else {
            Ok(Started::Response(HttpResponse::new(
                StatusCode::UNAUTHORIZED,
                "".into(),
            )))
        }
    }

    fn response(&self, req: &mut HttpRequest<State>, mut resp: HttpResponse) -> Result<Response> {
        Ok(Response::Done(resp))
    }
}
