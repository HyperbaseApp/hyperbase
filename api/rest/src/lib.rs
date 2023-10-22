use actix_web::{web, App, HttpServer};
use hb_config::ApiRestConfig;
use hb_db_scylladb::db::ScyllaDb;
use hb_hash_argon2::argon2::Argon2Hash;
use v1::v1_api;

mod v1;

pub struct Context {
    pub hash: HashCtx,
    pub db: DbCtx,
}

pub struct HashCtx {
    pub argon2: Argon2Hash,
}

pub struct DbCtx {
    pub scylladb: ScyllaDb,
}

pub async fn run(config: &ApiRestConfig, ctx: Context) {
    let addrs = format!("{}:{}", config.host(), config.port());

    let data = web::Data::new(ctx);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(web::scope("/api/rest/v1").configure(v1_api))
    })
    .bind(addrs)
    .unwrap()
    .run()
    .await
    .unwrap();
}
