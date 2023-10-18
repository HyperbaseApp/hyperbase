#[tokio::main]
async fn main() {
    let config_path =
        std::env::var("CONFIG_PATH").expect("CONFIG_PATH environment variable is required");
    let config = hb_config::new(&config_path);
    let scylla_db = hb_db_scylladb::new(config.db().scylla()).await;

    let mut apis = Vec::with_capacity(1);
    apis.push(hb_api_rest::run(config.api().rest()));

    futures::future::join_all(apis).await;

    println!("Hello, world!");
}
