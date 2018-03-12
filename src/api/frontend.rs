use actix_web::{fs, Application};

use api::State;
use api::state::AppState;

pub fn app(state: State) -> Application<State> {
    Application::with_state(state).handler(
        "/",
        fs::StaticFiles::new("src/frontend/static/", false).index_file("index.html"),
    )
}
