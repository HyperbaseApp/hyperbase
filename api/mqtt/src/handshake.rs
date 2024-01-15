use std::{str::FromStr, sync::Arc};

use hb_dao::token::TokenDao;
use ntex_mqtt::{v3, v5};
use uuid::Uuid;

use crate::{context::ApiMqttCtx, error_handler::ServerError, session::Session};

pub async fn handshake_v3(
    ctx: Arc<ApiMqttCtx>,
    handshake: v3::Handshake,
) -> Result<v3::HandshakeAck<Session>, ServerError> {
    if handshake.packet().username.is_some() && handshake.packet().password.is_some() {
        let client_id = handshake.packet().client_id.to_owned();
        let token_id = handshake.packet().username.as_ref().unwrap();
        let token = handshake.packet().password.as_ref().unwrap();

        let token_id = Uuid::from_str(token_id).map_err(|_| ServerError)?;
        let token_data = TokenDao::db_select(ctx.dao().db(), &token_id)
            .await
            .map_err(|_| ServerError)?;

        if token_data.token() == token {
            return Ok(handshake.ack(Session::new(&client_id, &token_id), false));
        } else {
            return Ok(handshake.identifier_rejected());
        }
    }
    Ok(handshake.bad_username_or_pwd())
}

pub async fn handshake_v5(
    ctx: Arc<ApiMqttCtx>,
    handshake: v5::Handshake,
) -> Result<v5::HandshakeAck<Session>, ServerError> {
    if handshake.packet().username.is_some() && handshake.packet().password.is_some() {
        let client_id = handshake.packet().client_id.to_owned();
        let token_id = handshake.packet().username.as_ref().unwrap();
        let token = handshake.packet().password.as_ref().unwrap();

        let token_id = Uuid::from_str(token_id).map_err(|_| ServerError)?;
        let token_data = TokenDao::db_select(ctx.dao().db(), &token_id)
            .await
            .map_err(|_| ServerError)?;

        if token_data.token() == token {
            return Ok(handshake.ack(Session::new(&client_id, &token_id)));
        } else {
            return Ok(handshake.fail_with(v5::codec::ConnectAck {
                reason_code: v5::codec::ConnectAckReason::ClientIdentifierNotValid,
                ..Default::default()
            }));
        }
    }
    Ok(handshake.fail_with(v5::codec::ConnectAck {
        reason_code: v5::codec::ConnectAckReason::BadUserNameOrPassword,
        ..Default::default()
    }))
}
