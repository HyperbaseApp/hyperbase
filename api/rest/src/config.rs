use actix_web::web;

use crate::service::{
    admin::admin_api, auth::auth_api, collection::collection_api, project::project_api,
    record::record_api, token::token_api,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.configure(auth_api)
        .configure(admin_api)
        .configure(token_api)
        .configure(project_api)
        .configure(collection_api)
        .configure(record_api);
}
