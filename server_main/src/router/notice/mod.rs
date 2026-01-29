pub mod insert;

use axum::Router;

pub fn notice_router() -> Router<crate::AppState> {
    insert::router()
}
