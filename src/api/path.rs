use bodyparser;
use iron::prelude::*;
use iron::status;
use serde_json;

use api::backend::BackendReqExt;
use api::error::APIError;
use flag::FlagPath;

const PATH_KEY: &'static str = "paths";

#[derive(Debug, Serialize, Deserialize)]
struct PathReqResp<'a> {
    pub app: &'a str,
    pub env: &'a str,
}

impl<'a> From<&'a FlagPath> for PathReqResp<'a> {
    fn from(path: &'a FlagPath) -> PathReqResp<'a> {
        PathReqResp {
            app: path.app.as_str(),
            env: path.env.as_str(),
        }
    }
}

impl<'a> From<PathReqResp<'a>> for FlagPath {
    fn from(reqresp: PathReqResp<'a>) -> FlagPath {
        FlagPath::new(reqresp.app, reqresp.env)
    }
}

pub fn create(req: &mut Request) -> IronResult<Response> {
    if let Ok(Some(raw_path)) = req.get::<bodyparser::Raw>() {
        let p_req: PathReqResp =
            serde_json::from_str(raw_path.as_str()).or(Err(APIError::FailedToParseBody))?;
        let path: FlagPath = p_req.into();

        let store = req.get_store().ok_or(APIError::FailedToAccessStore)?;

        let p = PATH_KEY.to_string();

        if let Ok(Some(_exists)) = store.get(&p, path.as_ref()) {
            Err(APIError::AlreadyExists)?
        }

        store
            .upsert(&p, path.as_ref(), &path)
            .and_then(|_| Ok(Response::with((status::Created, ""))))
            .map_err(|err| err.into())
    } else {
        Err(APIError::FailedToParseBody)?
    }
}

pub fn all(req: &mut Request) -> IronResult<Response> {
    let paths = req.get_store().ok_or(APIError::FailedToAccessStore)?;
    paths
        .get_all(&PATH_KEY.to_string())
        .and_then(|paths| {
            let stringy_paths = serde_json::to_string(&paths.values().collect::<Vec<&FlagPath>>())
                .or(Err(APIError::FailedToSerialize))?;
            Ok(Response::with((status::Ok, stringy_paths)))
        })
        .map_err(|err| err.into())
}
