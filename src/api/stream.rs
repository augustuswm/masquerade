use actix_web::http::ConnectionType;
use actix_web::{http, HttpResponse, State as ActixState};
use bytes::Bytes;
use futures::{future, stream, Future, Stream};
use serde_json;

use crate::api::error::APIError;
use crate::api::flag_req::FlagReq;
use crate::api::State;
use crate::flag::Flag;

const HEADER: &'static str = "event:data\n";

pub fn flag_stream(
    (flag_req, state): (FlagReq, ActixState<State>),
) -> Box<Future<Item = HttpResponse, Error = APIError>> {
    Box::new(
        state
            .flags()
            .update_sub()
            .map(|stream| {
                HttpResponse::Ok()
                    .header(http::header::CACHE_CONTROL, "no-cache")
                    .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                    .header(http::header::CONNECTION, "keep-alive")
                    .content_type("text/event-stream")
                    .content_encoding(http::ContentEncoding::Identity)
                    .no_chunking()
                    .connection_type(ConnectionType::KeepAlive)
                    // .force_close()
                    .streaming(
                        stream
                            .map_err(|err| {
                                println!("Failed to access store {:?}", err);
                                APIError::FailedToAccessStore(err)
                            })
                            .and_then(move |_| {
                                state
                                    .flags()
                                    .get_all(flag_req.path.clone())
                                    .map_err(APIError::FailedToAccessStore)
                                    .and_then(|flags| {
                                        let mut flag_list = flags.values().collect::<Vec<&Flag>>();
                                        flag_list
                                            .as_mut_slice()
                                            .sort_by(|&a, &b| a.key().cmp(b.key()));
                                        serde_json::to_string(&flag_list).map_err(|err| err.into())
                                    })
                                    .map(|json| HEADER.to_string() + "data:" + &json + "\n\n")
                                    .map(|event| Bytes::from(event))
                            }),
                    )
            })
            .or_else(|err| {
                println!("Error in generating stream {:?}", err);

                future::ok(
                    HttpResponse::Ok()
                        .header(http::header::CACHE_CONTROL, "no-cache")
                        .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                        .header(http::header::CONNECTION, "keep-alive")
                        .content_type("text/event-stream")
                        .content_encoding(http::ContentEncoding::Identity)
                        .no_chunking()
                        .connection_type(ConnectionType::KeepAlive)
                        .streaming(stream::empty::<Bytes, APIError>()),
                )
            }),
    )
}
