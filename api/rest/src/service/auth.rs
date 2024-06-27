use std::str::FromStr;

use actix_web::{http::StatusCode, web, HttpResponse, HttpResponseBuilder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use hb_api_websocket::message::{MessageKind as WebSocketMessageKind, Target as WebSocketTarget};
use hb_dao::{
    admin::AdminDao,
    admin_password_reset::AdminPasswordResetDao,
    collection::CollectionDao,
    log::{LogDao, LogKind},
    record::{RecordDao, RecordFilter, RecordFilters, RecordPagination},
    registration::RegistrationDao,
    token::TokenDao,
    value::ColumnValue,
};
use hb_mailer::MailPayload;
use hb_token_jwt::claim::{ClaimId, UserClaim};
use uuid::Uuid;
use validator::Validate;

use crate::{
    model::{
        auth::{
            AuthTokenResJson, ConfirmPasswordResetReqJson, ConfirmPasswordResetResJson,
            MqttAuthenticationReqJson, MqttAuthenticationResJson, MqttAuthorizationReqJson,
            MqttAuthorizationResJson, PasswordBasedReqJson, RegisterReqJson, RegisterResJson,
            RequestPasswordResetReqJson, RequestPasswordResetResJson, TokenBasedReqJson,
            VerifyRegistrationReqJson, VerifyRegistrationResJson,
        },
        log::LogResJson,
        Response,
    },
    util::ws_broadcast::websocket_broadcast,
    ApiRestCtx,
};

pub fn auth_api(cfg: &mut web::ServiceConfig) {
    cfg.route("/auth/token", web::get().to(token))
        .route("/auth/register", web::post().to(register))
        .route(
            "/auth/verify-registration",
            web::post().to(verify_registration),
        )
        .route("/auth/password-based", web::post().to(password_based))
        .route("/auth/token-based", web::post().to(token_based))
        .route(
            "/auth/mqtt_authentication",
            web::post().to(mqtt_authentication),
        )
        .route(
            "/auth/mqtt_authorization",
            web::post().to(mqtt_authorization),
        )
        .route(
            "/auth/request-password-reset",
            web::post().to(request_password_reset),
        )
        .route(
            "/auth/confirm-password-reset",
            web::post().to(confirm_password_reset),
        );
}

async fn token(ctx: web::Data<ApiRestCtx>, auth: BearerAuth) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    match token_claim.id() {
        ClaimId::Admin(id) => {
            if let Err(err) = AdminDao::db_select(ctx.dao().db(), id).await {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get admin data: {err}"),
                );
            }
        }
        ClaimId::Token(id, _) => {
            if let Err(err) = TokenDao::db_select(ctx.dao().db(), id).await {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get token data: {err}"),
                );
            }
        }
    }

    let token = match ctx.token().jwt().need_renew(&token_claim) {
        Ok(need) => {
            if need {
                match ctx.token().jwt().renew(&token_claim) {
                    Ok(token) => token,
                    Err(err) => {
                        return Response::error_raw(
                            &StatusCode::INTERNAL_SERVER_ERROR,
                            &err.to_string(),
                        )
                    }
                }
            } else {
                token.to_owned()
            }
        }
        Err(err) => {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
        }
    };

    Response::data(&StatusCode::OK, &None, &AuthTokenResJson::new(&token))
}

async fn register(ctx: web::Data<ApiRestCtx>, data: web::Json<RegisterReqJson>) -> HttpResponse {
    if !ctx.admin_registration() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Admin registration is disabled");
    }

    if let Err(err) = data.validate() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    let email = data.email().to_lowercase();

    if AdminDao::db_select_by_email(ctx.dao().db(), &email)
        .await
        .is_ok()
    {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Account has been registered");
    };

    let password_hash = match ctx
        .hash()
        .argon2()
        .hash_password(data.password().as_bytes())
    {
        Ok(hash) => hash,
        Err(err) => {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
        }
    };

    let registration_data = match RegistrationDao::db_select_by_email(ctx.dao().db(), &email).await
    {
        Ok(mut registration_data) => {
            registration_data.regenerate_code();
            if let Err(err) = registration_data.db_update(ctx.dao().db()).await {
                return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
            }
            registration_data
        }
        Err(_) => {
            let registration_data = RegistrationDao::new(&email, &password_hash.to_string());
            if let Err(err) = registration_data.db_insert(ctx.dao().db()).await {
                return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
            }
            registration_data
        }
    };

    if let Some(mailer) = ctx.mailer() {
        if let Err(err) = mailer
            .sender()
            .send(MailPayload::new(
                &email,
                "Registration Verification Code",
                &format!(
                "Your registration verification code is {}. This code will expire in {} seconds",
                registration_data.code(),
                ctx.registration_ttl()
            ),
            ))
            .await
        {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
        }
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &RegisterResJson::new(registration_data.id()),
    )
}

async fn verify_registration(
    ctx: web::Data<ApiRestCtx>,
    data: web::Json<VerifyRegistrationReqJson>,
) -> HttpResponse {
    if !ctx.admin_registration() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Admin registration is disabled");
    }

    let registration_data = match RegistrationDao::db_select(ctx.dao().db(), data.id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if data.code() != registration_data.code() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Wrong code");
    }

    let admin_data = AdminDao::new(registration_data.email(), registration_data.password_hash());

    if let Err(err) = admin_data.db_insert(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    if let Err(err) = registration_data.db_delete(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    if let Some(mailer) = ctx.mailer() {
        if let Err(err) = mailer
            .sender()
            .send(MailPayload::new(
                admin_data.email(),
                "Your Account Has Been Activated",
                "Your account has been successfully activated",
            ))
            .await
        {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
        }
    }

    Response::data(
        &StatusCode::CREATED,
        &None,
        &VerifyRegistrationResJson::new(admin_data.id()),
    )
}

async fn password_based(
    ctx: web::Data<ApiRestCtx>,
    data: web::Json<PasswordBasedReqJson>,
) -> HttpResponse {
    if let Err(err) = data.validate() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    let email = data.email().to_lowercase();

    let admin_data = match AdminDao::db_select_by_email(ctx.dao().db(), &email).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if let Err(err) = ctx
        .hash()
        .argon2()
        .verify_password(data.password(), admin_data.password_hash())
    {
        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    let token = match ctx.token().jwt().encode(&ClaimId::Admin(*admin_data.id())) {
        Ok(token) => token,
        Err(err) => {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
        }
    };

    Response::data(&StatusCode::OK, &None, &AuthTokenResJson::new(&token))
}

async fn token_based(
    ctx: web::Data<ApiRestCtx>,
    data: web::Json<TokenBasedReqJson>,
) -> HttpResponse {
    let token_data = match TokenDao::db_select(ctx.dao().db(), data.token_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_data.token() != data.token() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Token doesn't match");
    }

    let token = if let Some(collection_id) = data.collection_id() {
        if data.data().is_none() {
            return Response::error_raw(&StatusCode::BAD_REQUEST, "Field data must exist");
        }

        let collection_data = match CollectionDao::db_select(ctx.dao().db(), collection_id).await {
            Ok(data) => data,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

        let mut record_fields = HashSet::with_capacity(data.data().as_ref().unwrap().len() + 1);
        record_fields.insert("_id");
        let mut hashed_fields = HashMap::new();
        let mut record_filter_childs = Vec::with_capacity(data.data().as_ref().unwrap().len());

        for (field, props) in collection_data.schema_fields() {
            if *props.auth_column() {
                if let Some(value) = data.data().as_ref().unwrap().get(field) {
                    record_fields.insert(field.as_str());
                    if *props.hashed() {
                        if let Some(value_str) = value.as_str() {
                            hashed_fields.insert(field.as_str(), value_str);
                        } else {
                            return Response::error_raw(
                                &StatusCode::BAD_REQUEST,
                                &format!(
                                    "Field {field} must be of type string because it is hashed"
                                ),
                            );
                        }
                    } else {
                        let schema_field = match collection_data.schema_fields().get(field) {
                            Some(schema_field) => schema_field,
                            None => {
                                return Response::error_raw(
                                    &StatusCode::BAD_REQUEST,
                                    &format!("Field {field} doesn't exist in the collection"),
                                )
                            }
                        };
                        let column_value =
                            match ColumnValue::from_serde_json(schema_field.kind(), value) {
                                Ok(column_value) => column_value,
                                Err(err) => {
                                    return Response::error_raw(
                                        &StatusCode::BAD_REQUEST,
                                        &err.to_string(),
                                    )
                                }
                            };
                        record_filter_childs.push(RecordFilter::new(
                            &Some(field.to_owned()),
                            "=",
                            &Some(column_value),
                            &None,
                        ));
                    }
                } else {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        "Incorrect authentication data",
                    );
                }
            }
        }

        let record_filter = RecordFilters::new(&Vec::from([RecordFilter::new(
            &None,
            "AND",
            &None,
            &Some(RecordFilters::new(&record_filter_childs)),
        )]));

        let (records_data, total) = match RecordDao::db_select_many(
            ctx.dao().db(),
            &record_fields,
            &collection_data,
            &None,
            &record_filter,
            &Vec::new(),
            &Vec::new(),
            &RecordPagination::new(&Some(2)),
            &true,
        )
        .await
        {
            Ok(data) => data,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

        if total == 0 {
            return Response::error_raw(&StatusCode::BAD_REQUEST, "Authentication data not found");
        } else if total > 1 {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                "Multiple authentication data is found",
            );
        }

        for (field, value) in hashed_fields {
            if let Some(data_value) = records_data[0].get(field) {
                if let ColumnValue::String(data_value) = data_value {
                    if let Some(data_value) = data_value {
                        if ctx
                            .hash()
                            .argon2()
                            .verify_password(value, data_value)
                            .is_ok()
                        {
                            continue;
                        }
                    }
                } else {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        &format!("Field {field} must be of type string because it is hashed"),
                    );
                }
            }
            return Response::error_raw(&StatusCode::BAD_REQUEST, "Incorrect authentication data");
        }
        let record_id = if let Some(id) = records_data[0].id() {
            id
        } else {
            return Response::error_raw(
                &StatusCode::INTERNAL_SERVER_ERROR,
                "Can't parse authentication data id",
            );
        };

        match ctx.token().jwt().encode(&ClaimId::Token(
            *token_data.id(),
            Some(UserClaim::new(collection_id, record_id)),
        )) {
            Ok(token) => token,
            Err(err) => {
                return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
            }
        }
    } else {
        if !token_data.allow_anonymous() {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "Token is set to prevent anonymous login",
            );
        }
        match ctx
            .token()
            .jwt()
            .encode(&ClaimId::Token(*token_data.id(), None))
        {
            Ok(token) => token,
            Err(err) => {
                return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
            }
        }
    };

    Response::data(&StatusCode::OK, &None, &AuthTokenResJson::new(&token))
}

async fn mqtt_authentication(
    ctx: web::Data<ApiRestCtx>,
    data: web::Json<MqttAuthenticationReqJson>,
) -> HttpResponse {
    let mqtt_admin_credential = match ctx.mqtt_admin_credential() {
        Some(cred) => cred,
        None => {
            return HttpResponseBuilder::new(StatusCode::OK)
                .json(MqttAuthenticationResJson::new("allow", &true));
        }
    };

    if mqtt_admin_credential.username() == data.username() {
        if mqtt_admin_credential.password() == data.password() {
            return HttpResponseBuilder::new(StatusCode::OK)
                .json(MqttAuthenticationResJson::new("allow", &true));
        } else {
            return HttpResponseBuilder::new(StatusCode::BAD_REQUEST)
                .json(MqttAuthenticationResJson::new("deny", &false));
        }
    }

    let token_id = match Uuid::from_str(data.username()) {
        Ok(id) => id,
        Err(err) => {
            hb_log::error(
                None,
                &format!("Failed to parse token id '{}': {}", data.username(), err),
            );
            return HttpResponseBuilder::new(StatusCode::BAD_REQUEST)
                .json(MqttAuthenticationResJson::new("deny", &false));
        }
    };

    let token_data = match TokenDao::db_select(ctx.dao().db(), &token_id).await {
        Ok(data) => data,
        Err(err) => {
            hb_log::error(None, &format!("Failed to get token data: {err}"));
            return HttpResponseBuilder::new(StatusCode::BAD_REQUEST)
                .json(MqttAuthenticationResJson::new("deny", &false));
        }
    };

    if token_data.token() != data.password() {
        let err_msg = format!(
            "Token id '{}' doesn't match with token '{}'",
            token_id,
            data.password()
        );
        let log_data = LogDao::new(
            token_data.admin_id(),
            token_data.project_id(),
            &LogKind::Error,
            &format!("MQTT: Client is not authenticated: {err_msg}"),
        );

        tokio::spawn((|| async move {
            match log_data.db_insert(ctx.dao().db()).await {
                Ok(_) => {
                    if let Err(err) = websocket_broadcast(
                        ctx.websocket().handler(),
                        WebSocketTarget::Log,
                        None,
                        WebSocketMessageKind::InsertOne,
                        LogResJson::new(
                            log_data.id(),
                            log_data.created_at(),
                            log_data.kind().to_str(),
                            log_data.message(),
                        ),
                    ) {
                        hb_log::error(
                            None,
                            &format!(
                                "[ApiMqttClient] Error when serializing websocket data: {err}"
                            ),
                        );
                    }
                }
                Err(err) => hb_log::error(
                    None,
                    &format!("[ApiMqttClient] Error when inserting log data: {err}"),
                ),
            }
        })());
        hb_log::error(None, &err_msg);
        return HttpResponseBuilder::new(StatusCode::BAD_REQUEST)
            .json(MqttAuthenticationResJson::new("deny", &false));
    }

    tokio::spawn((|| async move {
        let log_data = LogDao::new(
            token_data.admin_id(),
            token_data.project_id(),
            &LogKind::Info,
            &format!(
                "MQTT: Client is authenticated using token id '{}' and token '{}'",
                token_data.id(),
                token_data.token()
            ),
        );
        match log_data.db_insert(ctx.dao().db()).await {
            Ok(_) => {
                if let Err(err) = websocket_broadcast(
                    ctx.websocket().handler(),
                    WebSocketTarget::Log,
                    None,
                    WebSocketMessageKind::InsertOne,
                    LogResJson::new(
                        log_data.id(),
                        log_data.created_at(),
                        log_data.kind().to_str(),
                        log_data.message(),
                    ),
                ) {
                    hb_log::error(
                        None,
                        &format!("[ApiMqttClient] Error when serializing websocket data: {err}"),
                    );
                }
            }
            Err(err) => hb_log::error(
                None,
                &format!("[ApiMqttClient] Error when inserting log data: {err}"),
            ),
        }
    })());

    HttpResponseBuilder::new(StatusCode::OK).json(MqttAuthenticationResJson::new("allow", &false))
}

async fn mqtt_authorization(
    ctx: web::Data<ApiRestCtx>,
    data: web::Json<MqttAuthorizationReqJson>,
) -> HttpResponse {
    let mqtt_admin_credential = match ctx.mqtt_admin_credential() {
        Some(cred) => cred,
        None => {
            return HttpResponseBuilder::new(StatusCode::OK)
                .json(MqttAuthorizationResJson::new("allow"));
        }
    };

    let mut is_allow = false;

    if data.action() == "subscribe" && mqtt_admin_credential.topic() == data.topic() {
        if mqtt_admin_credential.username() == data.username() {
            is_allow = true;
        }
    } else if data
        .topic()
        .chars()
        .all(|c| c.is_alphabetic() || c == '/' || c == '-')
    {
        is_allow = true
    }

    tokio::spawn((|| async move {
        let token_id = match Uuid::from_str(data.username()) {
            Ok(id) => id,
            Err(err) => {
                hb_log::error(
                    None,
                    &format!("Failed to parse token id '{}': {}", data.username(), err),
                );
                return;
            }
        };
        let token_data = match TokenDao::db_select(ctx.dao().db(), &token_id).await {
            Ok(data) => data,
            Err(err) => {
                hb_log::error(
                    None,
                    &format!(
                        "Failed to get token data with id '{}': {}",
                        data.username(),
                        err
                    ),
                );
                return;
            }
        };
        let (kind, message) = match is_allow {
            true => (
                LogKind::Info,
                format!(
                    "MQTT: Client is authorized using token id '{}'",
                    token_data.id(),
                ),
            ),
            false => (
                LogKind::Error,
                format!(
                    "MQTT: Client is not authorized: Token id '{}' Topic '{}' Action '{}'",
                    token_data.id(),
                    data.topic(),
                    data.action()
                ),
            ),
        };
        let log_data = LogDao::new(
            token_data.admin_id(),
            token_data.project_id(),
            &kind,
            &message,
        );
        match log_data.db_insert(ctx.dao().db()).await {
            Ok(_) => {
                if let Err(err) = websocket_broadcast(
                    ctx.websocket().handler(),
                    WebSocketTarget::Log,
                    None,
                    WebSocketMessageKind::InsertOne,
                    LogResJson::new(
                        log_data.id(),
                        log_data.created_at(),
                        log_data.kind().to_str(),
                        log_data.message(),
                    ),
                ) {
                    hb_log::error(
                        None,
                        &format!("[ApiMqttClient] Error when serializing websocket data: {err}"),
                    );
                }
            }
            Err(err) => hb_log::error(
                None,
                &format!("[ApiMqttClient] Error when inserting log data: {err}"),
            ),
        }
    })());

    if is_allow {
        HttpResponseBuilder::new(StatusCode::OK).json(MqttAuthorizationResJson::new("allow"))
    } else {
        HttpResponseBuilder::new(StatusCode::OK).json(MqttAuthorizationResJson::new("deny"))
    }
}

async fn request_password_reset(
    ctx: web::Data<ApiRestCtx>,
    data: web::Json<RequestPasswordResetReqJson>,
) -> HttpResponse {
    if let Err(err) = data.validate() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
    };

    let email = data.email().to_lowercase();

    let admin_data = match AdminDao::db_select_by_email(ctx.dao().db(), &email).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let password_reset_data = AdminPasswordResetDao::new(admin_data.id());

    if let Err(err) = password_reset_data.db_insert(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    if let Some(mailer) = ctx.mailer() {
        if let Err(err)= mailer
        .sender()
        .send(MailPayload::new(
            &email,
            "Request Password Reset Verification Code",
            &format!(
                "Your request password reset verification code is {}. This code will expire in {} seconds",
                password_reset_data.code(),
                ctx.reset_password_ttl()
            ),
        )).await {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());

        }
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &RequestPasswordResetResJson::new(password_reset_data.id()),
    )
}

async fn confirm_password_reset(
    ctx: web::Data<ApiRestCtx>,
    data: web::Json<ConfirmPasswordResetReqJson>,
) -> HttpResponse {
    let password_reset_data =
        match AdminPasswordResetDao::db_select(ctx.dao().db(), data.id()).await {
            Ok(data) => data,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

    if data.code() != password_reset_data.code() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Wrong code");
    }

    let mut admin_data =
        match AdminDao::db_select(ctx.dao().db(), password_reset_data.admin_id()).await {
            Ok(data) => data,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

    let password_hash = match ctx
        .hash()
        .argon2()
        .hash_password(data.password().as_bytes())
    {
        Ok(hash) => hash,
        Err(err) => {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
        }
    };

    admin_data.set_password_hash(&password_hash.to_string());

    if let Err(err) = admin_data.db_update(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    if let Some(mailer) = ctx.mailer() {
        if let Err(err) = mailer
            .sender()
            .send(MailPayload::new(
                admin_data.email(),
                "Your Password Has Been Reset Successfully",
                "Your account password has been successfully changed",
            ))
            .await
        {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
        }
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &ConfirmPasswordResetResJson::new(admin_data.id()),
    )
}
