use actix_web::{http, server, App, HttpRequest, HttpResponse, Error};
use bytes::Bytes;
use futures::{task, Async, Poll, Stream};
use serde_json;
use tokio::timer::Interval;
use uuid::Uuid;

use std::time::{Duration, Instant};

use api::error::APIError;
use api::flag_req::FlagReq;
use api::State;
use api::state::FlagStore;
use flag;
use flag::{Flag, FlagPath};


const HEADER: &'static str = "event:data\n";

struct FlagStream {
    id: String,
    path: FlagPath,
    state: State,
    last_seen: Instant,
    subbed: bool,
}

impl FlagStream {
    fn get_store_payload(&self) -> Poll<Option<Bytes>, Error> {
        self.state.flags().get_all(&self.path).and_then(|flags| {
            let mut flag_list = flags.values().collect::<Vec<&Flag>>();
            flag_list.as_mut_slice().sort_by(|&a, &b| a.key().cmp(b.key()));
            serde_json::to_string(&flag_list).map_err(|err| err.into())
        }).map(|json| {
            HEADER.to_string() + "data:" + &json + "\n\n"
        }).map(|event| {
            Bytes::from(event)
        }).map(|bytes| {
            Async::Ready(Some(bytes))
        }).or_else(|_| {
            Ok(Async::NotReady)
        })
    }

    fn poll_store(&mut self) -> Poll<Option<Bytes>, Error> {
        match self.state.flags().updated_at() {
            Ok(updated) => {
                if updated > self.last_seen {
                    self.last_seen = updated;
                    self.get_store_payload()
                } else {
                    Ok(Async::NotReady)
                }
            },
            Err(err) => {
                Ok(Async::Ready(None))
            }
        }
    }
}

impl Drop for FlagStream {
    fn drop(&mut self) {
        self.state.flags().unsub(self.id.as_str(), &self.path)
    }
}

impl Stream for FlagStream {
    type Item = Bytes;
    type Error = Error;
                            
    fn poll(&mut self) -> Poll<Option<Bytes>, Error> {
        if !self.subbed {
            self.subbed = self.state.flags().sub(self.id.as_str(), &self.path, task::current());
            self.get_store_payload()
        } else {
            self.poll_store()
        }
    }
}

pub fn flag_stream(req: HttpRequest<State>) -> Result<HttpResponse, APIError> {
    let flag_req = FlagReq::from_req(&req)?;

    let stream = FlagStream {
        id: Uuid::new_v4().to_string(),
        path: flag_req.path,
        state: req.state().clone(),
        last_seen: Instant::now(),
        subbed: false,
    };

    Ok(HttpResponse::Ok()
        .header(http::header::CACHE_CONTROL, "no-cache")
        .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .content_type("text/event-stream")
        .content_encoding(http::ContentEncoding::Identity)
        .no_chunking()
        .force_close()
        .streaming(stream))
}