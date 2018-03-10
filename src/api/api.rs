use bodyparser;
use iron::Chain;
use iron::IronError;
use iron::status as RespStatus;
use logger::Logger;
use persistent::Read;
use router::Router;

use api::error::APIError;
use api::flag;
use api::path;
use api::status;
use api::backend;
use error::BannerError;
use flag::{Flag, FlagPath};
use store::ThreadedStore;

const MAX_BODY_LENGTH: usize = 1024 * 1024 * 10;

macro_rules! rest {
	( $router:expr, $handler:ident, $base:expr, $path:expr, $key:expr ) => {
		{
			let base_path = format!("{}/{}/", $base, $path);
			let indv_path = format!("{}/{}/:{}/", $base, $path, $key);
            let all_path = format!("{}/{}s/", $base, $path);

			$router.post(base_path, $handler::create, format!("create_{}", $path));
		    $router.get(indv_path.as_str(), $handler::read, format!("read_{}", $path));
		    $router.post(indv_path.as_str(), $handler::update, format!("update_{}", $path));
		    $router.delete(indv_path.as_str(), $handler::delete, format!("delete_{}", $path));
            $router.get(all_path.as_str(), $handler::all, format!("all_{}s", $path));
		}
	};
}

pub fn v1<S, T>(paths: S, flags: T) -> Chain
where
    S: ThreadedStore<String, FlagPath, Error = BannerError> + 'static,
    T: ThreadedStore<FlagPath, Flag, Error = BannerError> + 'static,
{
    let mut router = Router::new();

    // Health check
    router.get("/status/", status::handler, "status");

    // Path index
    router.post("/path/", path::create, "create_path");
    router.get("/paths/", path::all, "all_paths");

    // Flag handling
    rest!(router, flag, "/:app/:env", "flag", "key");

    let mut chain = Chain::new(router);

    let (logger_before, logger_after) = Logger::new(None);

    chain.link_before(logger_before);
    chain.link_before(backend::BackendMiddleware::new(paths));
    chain.link_before(backend::BackendMiddleware::new(flags));
    chain.link_before(Read::<bodyparser::MaxBodyLength>::one(MAX_BODY_LENGTH));
    chain.link_after(logger_after);

    chain
}

impl From<BannerError> for IronError {
    fn from(b_err: BannerError) -> IronError {
        IronError::new(b_err, (RespStatus::InternalServerError, ""))
    }
}

impl From<APIError> for IronError {
    fn from(a_err: APIError) -> IronError {
        let status = a_err.status();
        let msg = a_err.to_string();
        IronError::new(BannerError::APIError(a_err), (status, msg))
    }
}

impl From<APIError> for BannerError {
    fn from(a_err: APIError) -> BannerError {
        BannerError::APIError(a_err)
    }
}
