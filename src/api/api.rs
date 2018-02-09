use iron::IronError;
use iron::status as RespStatus;
use router::Router;

use api::error::APIError;
use api::flag;
use api::status;
use error::BannerError;

macro_rules! rest {
	( $router:expr, $handler:ident, $base:expr, $path:expr, $key:expr ) => {
		{
			let base_path = format!("{}/{}/", $base, $path);
			let indv_path = format!("{}/{}/:{}/", $base, $path, $key);

			$router.post(base_path, $handler::create, format!("create_{}", $path));
		    $router.get(indv_path.as_str(), $handler::read, format!("read_{}", $path));
		    $router.post(indv_path.as_str(), $handler::update, format!("update_{}", $path));
		    $router.delete(indv_path.as_str(), $handler::delete, format!("delete_{}", $path));
		}
	};
}

pub fn v1() -> Router {
    let mut router = Router::new();

    // Health check
    router.get("/status/", status::handler, "status");

    // Flag handling
    rest!(router, flag, "/:app/:env", "flag", "key");
    router.get("/:app/:env/flags/", flag::all, "all_flags");

    router
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
