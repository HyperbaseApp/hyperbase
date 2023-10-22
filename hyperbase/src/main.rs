use hb_api_rest::{Context, DbCtx, HashCtx};
use hb_db_scylladb::db::ScyllaDb;
use hb_hash_argon2::argon2::Argon2Hash;

#[tokio::main]
async fn main() {
    let config_path =
        std::env::var("CONFIG_PATH").expect("CONFIG_PATH environment variable is required");
    let config = hb_config::new(&config_path);

    let argon2_hash = Argon2Hash::new(
        config.hash().argon2().algorithm(),
        config.hash().argon2().version(),
        config.hash().argon2().salt(),
    );
    let scylla_db = ScyllaDb::new(
        config.db().scylla().host(),
        config.db().scylla().port(),
        config.db().scylla().replication_factor(),
        config.db().scylla().temp_ttl(),
    )
    .await;

    let mut apis = Vec::with_capacity(1);
    apis.push(hb_api_rest::run(
        config.api().rest(),
        Context {
            hash: HashCtx {
                argon2: argon2_hash,
            },
            db: DbCtx {
                scylladb: scylla_db,
            },
        },
    ));

    futures::future::join_all(apis).await;

    println!("Hello, world!");
}
