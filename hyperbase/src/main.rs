use hb_api_rest::{
    context::{Context as ApiRestCtx, DaoCtx, HashCtx, MailerCtx, TokenCtx},
    ApiRestServer,
};
use hb_db_scylladb::db::ScyllaDb;
use hb_hash_argon2::argon2::Argon2Hash;
use hb_mailer::Mailer;
use hb_token_jwt::token::JwtToken;

mod config_path;

#[tokio::main]
async fn main() {
    let config_path = config_path::get();
    let config = hb_config::new(&config_path);

    hb_log::init(config.log().display_level(), config.log().level_filter());

    hb_log::info(Some("🚀"), &"Starting Hyperbase");

    let argon2_hash = Argon2Hash::new(
        config.hash().argon2().algorithm(),
        config.hash().argon2().version(),
        config.hash().argon2().salt(),
    );
    let jwt_token = JwtToken::new(
        config.token().jwt().secret(),
        config.token().jwt().expiry_duration(),
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
        config.db().scylla().temporary_ttl(),
    )
    .await;

    let api_rest_server = ApiRestServer::new(
        config.api().rest().host(),
        config.api().rest().port(),
        ApiRestCtx {
            hash: HashCtx {
                argon2: argon2_hash,
            },
            token: TokenCtx { jwt: jwt_token },
            mailer: MailerCtx {
                sender: mailer_sender,
            },
            dao: DaoCtx {
                db: hb_dao::Db::ScyllaDb(scylla_db),
            },
            verification_code_ttl: *config.db().scylla().temporary_ttl(),
        },
    );

    tokio::try_join!(mailer.run(), api_rest_server.run()).unwrap();

    hb_log::info(Some("👋"), &"Hyperbase is turned off");
}
