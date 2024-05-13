use std::sync::Arc;

use hb_api_internal_gossip::{ApiInternalGossip, InternalBroadcast};
use hb_api_mqtt::{
    context::{ApiMqttCtx, ApiMqttDaoCtx, ApiMqttWsCtx},
    ApiMqttClient,
};
use hb_api_rest::{
    context::{
        ApiRestCtx, ApiRestDaoCtx, ApiRestHashCtx, ApiRestMailerCtx, ApiRestTokenCtx, ApiRestWsCtx,
        MqttAdminCredential,
    },
    ApiRestServer,
};
use hb_api_websocket::{context::ApiWebSocketCtx, ApiWebSocketServer};
use hb_dao::Db;
use hb_db_mysql::db::MysqlDb;
use hb_db_postgresql::db::PostgresDb;
use hb_db_scylladb::db::ScyllaDb;
use hb_db_sqlite::db::SqliteDb;
use hb_hash_argon2::argon2::Argon2Hash;
use hb_mailer::Mailer;
use hb_token_jwt::token::JwtToken;
use tokio_util::sync::CancellationToken;

mod config_path;

#[tokio::main]
async fn main() {
    let config_path = config_path::get();
    let config = hb_config::from_path(&config_path);

    hb_log::init(config.log().display_level(), config.log().level_filter());

    hb_log::info(Some("🚀"), "[Hyperbase] Starting");

    let argon2_hash = Argon2Hash::new(
        config.hash().argon2().algorithm(),
        config.hash().argon2().version(),
        config.hash().argon2().salt(),
    );

    let jwt_token = JwtToken::new(
        config.token().jwt().secret(),
        config.token().jwt().expiry_duration(),
    );

    let (mailer, mailer_sender) = match config.mailer() {
        Some(config_mailer) => {
            let (mailer, mailer_sender) = Mailer::new(
                config_mailer.smtp_host(),
                config_mailer.smtp_username(),
                config_mailer.smtp_password(),
                config_mailer.sender_name(),
                config_mailer.sender_email(),
            );
            (Some(mailer), Some(mailer_sender))
        }
        None => (None, None),
    };

    let db = if let Some(scylla) = config.db().scylla() {
        Arc::new(Db::ScyllaDb(
            ScyllaDb::new(
                scylla.user(),
                scylla.password(),
                scylla.host(),
                scylla.port(),
                scylla.replication_factor(),
                scylla.prepared_statement_cache_size(),
                config.auth().registration_ttl(),
                config.auth().reset_password_ttl(),
                config.log().db_ttl(),
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
                &i64::from(*config.log().db_ttl()),
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
                &i64::from(*config.log().db_ttl()),
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
                &i64::from(*config.log().db_ttl()),
            )
            .await,
        ))
    } else {
        hb_log::panic(None, "[Hyperbase] No database configuration is specified");
        return;
    };
    if config
        .db()
        .option()
        .as_ref()
        .is_some_and(|opt| opt.refresh_change().is_some_and(|refresh| refresh))
    {
        if let Err(err) = db.init().await {
            hb_log::panic(
                None,
                format!("[Hyperbase] Initilizing database failed: {err}"),
            );
        }
    }

    let mut api_internal_gossip = None;
    let mut internal_broadcast = None;

    if let Some(config_internal) = config.api().internal() {
        if let Some(config_gossip) = config_internal.gossip() {
            let gossip_api = ApiInternalGossip::new(
                config_gossip.host(),
                config_gossip.port(),
                db.clone(),
                config_gossip.peers(),
                config_gossip.view_size(),
                config_gossip.actions_size(),
            )
            .await;
            api_internal_gossip = Some(gossip_api.0);
            internal_broadcast = Some(
                InternalBroadcast::new(
                    gossip_api.1,
                    db.clone(),
                    config_gossip.host(),
                    config_gossip.port(),
                )
                .await,
            );
        }
    }

    let (api_websocket_server, websocket_handler, websocket_publisher) = ApiWebSocketServer::new(
        ApiWebSocketCtx::new(db.clone()),
        config.api().websocket().heartbeat_interval(),
        config.api().websocket().client_timeout(),
    );

    let api_rest_server = ApiRestServer::new(
        config.app().mode(),
        config.api().rest().host(),
        config.api().rest().port(),
        config.api().rest().allowed_origin(),
        ApiRestCtx::new(
            ApiRestHashCtx::new(argon2_hash),
            ApiRestTokenCtx::new(jwt_token),
            match mailer_sender {
                Some(mailer_sender) => Some(ApiRestMailerCtx::new(mailer_sender)),
                None => None,
            },
            ApiRestDaoCtx::new(db.clone()),
            ApiRestWsCtx::new(websocket_handler),
            match config.api().mqtt() {
                Some(config_mqtt) => Some(MqttAdminCredential::new(
                    config_mqtt.username(),
                    config_mqtt.password(),
                    config_mqtt.topic(),
                )),
                None => None,
            },
            internal_broadcast.clone(),
            *config.auth().admin_registration(),
            *config.auth().access_token_length(),
            *config.auth().registration_ttl(),
            *config.auth().reset_password_ttl(),
            config.bucket().path().to_owned(),
        ),
    );

    let api_mqtt_client = match config.api().mqtt() {
        Some(config_mqtt) => Some(ApiMqttClient::new(
            config_mqtt.host(),
            config_mqtt.port(),
            config_mqtt.topic(),
            config_mqtt.username(),
            config_mqtt.password(),
            config_mqtt.channel_capacity(),
            config_mqtt.timeout(),
            ApiMqttCtx::new(
                ApiMqttDaoCtx::new(db),
                ApiMqttWsCtx::new(websocket_publisher),
                internal_broadcast,
            ),
        )),
        None => None,
    };

    let cancel_token = CancellationToken::new();

    match tokio::try_join!(
        match api_internal_gossip {
            Some(api_internal_gossip) => api_internal_gossip.run(cancel_token.clone()),
            None => ApiInternalGossip::run_none(),
        },
        match mailer {
            Some(mailer) => mailer.run(cancel_token.clone()),
            None => Mailer::run_none(),
        },
        api_rest_server.run(cancel_token.clone()),
        match api_mqtt_client {
            Some(api_mqtt_client) => api_mqtt_client.run(cancel_token.clone()),
            None => ApiMqttClient::run_none(),
        },
        api_websocket_server.run(cancel_token.clone())
    ) {
        Ok(_) => hb_log::info(Some("👋"), "[Hyperbase] Turned off"),
        Err(err) => {
            hb_log::warn(None, "[Hyperbase] Shutting down all running components");
            cancel_token.cancel();
            hb_log::warn(
                Some("👋"),
                format!("[Hyperbase] Turned off with error: {err}"),
            );
        }
    }
}
