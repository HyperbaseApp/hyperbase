use hb_api_rest::context::{Context, DbCtx, HashCtx, MailerCtx};
use hb_db_scylladb::db::ScyllaDb;
use hb_hash_argon2::argon2::Argon2Hash;
use hb_mailer::Mailer;

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
    let (mailer, mailer_sender) = Mailer::new(
        config.mailer().smtp_host(),
        config.mailer().smtp_username(),
        config.mailer().smtp_password(),
        config.mailer().sender_name(),
        config.mailer().sender_email(),
    );
    let scylla_db = ScyllaDb::new(
        config.db().scylla().host(),
        config.db().scylla().port(),
        config.db().scylla().replication_factor(),
        config.db().scylla().temp_ttl(),
    )
    .await;

    tokio::join!(
        mailer.run(),
        hb_api_rest::run(
            config.api().rest(),
            Context {
                hash: HashCtx {
                    argon2: argon2_hash,
                },
                mailer: MailerCtx {
                    sender: mailer_sender,
                },
                db: DbCtx {
                    scylladb: scylla_db,
                },
            },
        )
    );

    println!("Hello, world!");
}
