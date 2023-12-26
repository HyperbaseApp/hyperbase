use actix_web::{http::StatusCode, web, HttpResponse};

use crate::model::Response;

pub fn root_api(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(root))
        .route("/api", web::get().to(root))
        .route("/api/rest", web::get().to(root));
}

async fn root() -> HttpResponse {
    Response::data(&StatusCode::OK, &None, "Hyperbase is running")
}
