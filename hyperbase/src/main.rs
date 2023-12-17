use hb_api_rest::{
    context::{ApiRestCtx, DaoCtx, HashCtx, MailerCtx, TokenCtx},
    ApiRestServer,
};
use hb_dao::Db;
use hb_db_mysql::db::MysqlDb;
use hb_db_postgresql::db::PostgresDb;
use hb_db_scylladb::db::ScyllaDb;
use hb_db_sqlite::db::SqliteDb;
use hb_hash_argon2::argon2::Argon2Hash;
use hb_mailer::Mailer;
use hb_token_jwt::token::JwtToken;

mod config_path;

#[tokio::main]
async fn main() {
    let config_path = config_path::get();
    let config = hb_config::new(&config_path);

    hb_log::init(config.log().display_level(), config.log().level_filter());

    hb_log::info(Some("🚀"), "Hyperbase: Starting");

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
    let db = if let Some(scylla) = config.db().scylla() {
        Db::ScyllaDb(
            ScyllaDb::new(
                scylla.host(),
                scylla.port(),
                scylla.replication_factor(),
                scylla.prepared_statement_cache_size(),
                config.auth().registration_ttl(),
                config.auth().reset_password_ttl(),
            )
            .await,
        )
    } else if let Some(postgres) = config.db().postgres() {
        Db::PostgresqlDb(
            PostgresDb::new(
                postgres.user(),
                postgres.password(),
                postgres.host(),
                postgres.port(),
                postgres.db_name(),
                postgres.max_connections(),
                &i64::from(*config.auth().registration_ttl()),
                &i64::from(*config.auth().reset_password_ttl()),
            )
            .await,
        )
    } else if let Some(mysql) = config.db().mysql() {
        Db::MysqlDb(
            MysqlDb::new(
                mysql.user(),
                mysql.password(),
                mysql.host(),
                mysql.port(),
                mysql.db_name(),
                mysql.max_connections(),
                &i64::from(*config.auth().registration_ttl()),
                &i64::from(*config.auth().reset_password_ttl()),
            )
            .await,
        )
    } else if let Some(sqlite) = config.db().sqlite() {
        Db::SqliteDb(
            SqliteDb::new(
                sqlite.path(),
                sqlite.max_connections(),
                &i64::from(*config.auth().registration_ttl()),
                &i64::from(*config.auth().reset_password_ttl()),
            )
            .await,
        )
    } else {
        panic!("No database configuration is specified")
    };

    let api_rest_server = ApiRestServer::new(
        config.api().rest().host(),
        config.api().rest().port(),
        ApiRestCtx::new(
            HashCtx::new(argon2_hash),
            TokenCtx::new(jwt_token),
            MailerCtx::new(mailer_sender),
            DaoCtx::new(db),
            *config.auth().registration_ttl(),
            *config.auth().reset_password_ttl(),
            *config.auth().access_token_length(),
        ),
    );

    tokio::try_join!(mailer.run(), api_rest_server.run()).unwrap();

    hb_log::info(Some("👋"), "Hyperbase: turned off");
}
