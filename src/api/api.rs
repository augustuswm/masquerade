use router::Router;

use api::app;
use api::env;
use api::flag;
use api::status;

macro_rules! rest {
	( $router:expr, $handler:expr, $base:expr, $path:expr, $key:expr ) => {
		{
			let base_path = format!("{}/{}/", $base, $path);
			let indv_path = format!("{}/{}/:{}/", $base, $path, $key);

			$router.post(base_path, $handler, format!("create_{}", $path));
		    $router.get(indv_path.as_str(), $handler, format!("get_{}", $path));
		    $router.post(indv_path.as_str(), $handler, format!("update_{}", $path));
		    $router.delete(indv_path.as_str(), $handler, format!("delete_{}", $path));
		}
	};
}

pub fn v1() -> Router {
    let mut router = Router::new();

    // Health check
    router.get("/status/", status::handler, "status");

    // App handling
    rest!(router, app::handler, "", "app", "app");

    // Env handling
    rest!(router, env::handler, "/app/:app", "env", "env");

    // Flag handling
    rest!(router, flag::handler, "/app/:app/env/:env", "flag", "key");
    router.get("/app/:app/:env/flags/", flag::all_handler, "all_flags");

    router
}
