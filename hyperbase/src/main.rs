use std::sync::Arc;

use hb_api_mqtt::{
    context::{ApiMqttCtx, ApiMqttDaoCtx},
    ApiMqttClient,
};
use hb_api_rest::{
    context::{ApiRestCtx, ApiRestDaoCtx, ApiRestHashCtx, ApiRestMailerCtx, ApiRestTokenCtx},
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
    mailer.run();

    let db = if let Some(scylla) = config.db().scylla() {
        Arc::new(Db::ScyllaDb(
            ScyllaDb::new(
                scylla.host(),
                scylla.port(),
                scylla.replication_factor(),
                scylla.prepared_statement_cache_size(),
                config.auth().registration_ttl(),
                config.auth().reset_password_ttl(),
            )
            .await,
        ))
    } else if let Some(postgres) = config.db().postgres() {
        Arc::new(Db::PostgresqlDb(
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
        ))
    } else if let Some(mysql) = config.db().mysql() {
        Arc::new(Db::MysqlDb(
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
        ))
    } else if let Some(sqlite) = config.db().sqlite() {
        Arc::new(Db::SqliteDb(
            SqliteDb::new(
                sqlite.path(),
                sqlite.max_connections(),
                &i64::from(*config.auth().registration_ttl()),
                &i64::from(*config.auth().reset_password_ttl()),
            )
            .await,
        ))
    } else {
        panic!("No database configuration is specified")
    };

    let api_rest_server = ApiRestServer::new(
        config.api().rest().host(),
        config.api().rest().port(),
        ApiRestCtx::new(
            ApiRestHashCtx::new(argon2_hash),
            ApiRestTokenCtx::new(jwt_token),
            ApiRestMailerCtx::new(mailer_sender),
            ApiRestDaoCtx::new(db.clone()),
            *config.auth().admin_registration(),
            *config.auth().access_token_length(),
            *config.auth().registration_ttl(),
            *config.auth().reset_password_ttl(),
            config.bucket().path().to_owned(),
        ),
    );

    let api_mqtt_client = ApiMqttClient::new(
        config.api().mqtt().host(),
        config.api().mqtt().port(),
        config.api().mqtt().topic(),
        config.api().mqtt().channel_capacity(),
        ApiMqttCtx::new(ApiMqttDaoCtx::new(db)),
    );

    tokio::try_join!(api_rest_server.run(), api_mqtt_client.run()).unwrap();

    hb_log::info(Some("👋"), "Hyperbase: turned off");
}
