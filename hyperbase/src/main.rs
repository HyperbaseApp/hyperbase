use hb_api_rest::Context;

#[tokio::main]
async fn main() {
    let config_path =
        std::env::var("CONFIG_PATH").expect("CONFIG_PATH environment variable is required");
    let config = hb_config::new(&config_path);
    let argon2_hash = hb_hash_argon2::new(config.hash().argon2());
    let scylla_db = hb_db_scylladb::new(config.db().scylla()).await;

    let mut apis = Vec::with_capacity(1);
    apis.push(hb_api_rest::run(
        config.api().rest(),
        Context { argon2_hash },
    ));

    futures::future::join_all(apis).await;

    println!("Hello, world!");
}
