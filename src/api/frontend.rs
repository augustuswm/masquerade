use actix_web::{fs, Application};
use actix_web::middleware::Logger;

use api::State;

pub fn app(state: State) -> Application<State> {
    Application::with_state(state)
        .middleware(Logger::default())
        .handler(
            "/",
            fs::StaticFiles::new("src/frontend/static/", false).index_file("index.html"),
        )
}
