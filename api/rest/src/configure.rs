use actix_web::web;

use crate::service::{
    admin::admin_api, auth::auth_api, bucket::bucket_api, collection::collection_api,
    file::file_api, info::info_api, project::project_api, record::record_api, root::root_api,
    token::token_api,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(root_api).service(
        web::scope("/api/rest")
            .configure(info_api)
            .configure(auth_api)
            .configure(admin_api)
            .configure(token_api)
            .configure(project_api)
            .configure(collection_api)
            .configure(record_api)
            .configure(bucket_api)
            .configure(file_api),
    );
}
