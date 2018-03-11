use actix_web::Application;

use std::sync::Arc;

use api::state::AppState;

pub fn app(state: Arc<AppState>) -> Application<Arc<AppState>> {
    Application::with_state(state).prefix("/api/v1")
}
