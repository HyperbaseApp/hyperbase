use actix_web::web;

use self::service::{
    admin::admin_api, auth::auth_api, collection::collection_api, project::project_api,
    record::record_api,
};

pub mod model;
mod service;

pub fn v1_api(cfg: &mut web::ServiceConfig) {
    cfg.configure(auth_api)
        .configure(admin_api)
        .configure(project_api)
        .configure(collection_api)
        .configure(record_api);
}
