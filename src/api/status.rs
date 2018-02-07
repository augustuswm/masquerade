use iron::prelude::*;
use iron::status;
#[cfg(feature = "redis-backend")]
use redis::Client;
use serde_json;

use api::backend::BackendReqExt;

#[derive(Debug, Serialize, Deserialize)]
struct Status {
    pub backend: bool,
}

pub fn handler(req: &mut Request) -> IronResult<Response> {
    println!("{:?}", req.get_store());
    #[cfg(feature = "redis-backend")]
    let status = Status {
        backend: redis_health(),
    };
    #[cfg(not(feature = "redis-backend"))]
    let status = Status { backend: true };

    serde_json::to_string(&status)
        .map(|resp_body| Response::with((status::Ok, resp_body.as_str())))
        .or(Ok(Response::with((status::InternalServerError, ""))))
}

#[cfg(feature = "redis-backend")]
fn redis_health() -> bool {
    let url = format!("redis://{}:{}", "localhost", 6379);
    Client::open(url.as_str())
        .map(|client| client.get_connection().is_ok())
        .unwrap_or(false)
}
