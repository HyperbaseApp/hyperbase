use std::str::FromStr;

use actix_web::{http::StatusCode, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use ahash::HashSet;
use hb_dao::{
    admin::AdminDao,
    admin_password_reset::AdminPasswordResetDao,
    collection::CollectionDao,
    record::{RecordDao, RecordFilter, RecordFilters, RecordPagination},
    register::RegistrationDao,
    token::TokenDao,
    value::ColumnValue,
};
use hb_mailer::MailPayload;
use hb_token_jwt::{claim::UserClaim, kind::JwtTokenKind};
use uuid::Uuid;
use validator::Validate;

use crate::{
    model::{
        auth::{
            AuthTokenResJson, ConfirmPasswordResetReqJson, ConfirmPasswordResetResJson,
            PasswordBasedReqJson, RegisterReqJson, RegisterResJson, RequestPasswordResetReqJson,
            RequestPasswordResetResJson, TokenBasedReqJson, VerifyRegistrationReqJson,
            VerifyRegistrationResJson,
        },
        Response,
    },
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

    match token_claim.kind() {
        JwtTokenKind::Admin => {
            if let Err(err) = AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                );
            }
        }
        JwtTokenKind::Token => {
            if let Err(err) = TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
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

    if AdminDao::db_select_by_email(ctx.dao().db(), data.email())
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

    let registration_data = match RegistrationDao::db_select_by_email(ctx.dao().db(), data.email())
        .await
    {
        Ok(mut registration_data) => {
            registration_data.regenerate_code();
            if let Err(err) = registration_data.db_update(ctx.dao().db()).await {
                return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
            }
            registration_data
        }
        Err(_) => {
            let registration_data = RegistrationDao::new(data.email(), &password_hash.to_string());
            if let Err(err) = registration_data.db_insert(ctx.dao().db()).await {
                return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
            }
            registration_data
        }
    };

    if let Err(err) = ctx.mailer().sender().send(MailPayload::new(
        data.email(),
        "Registration Verification Code",
        &format!(
            "Your registration verification code is {}. This code will expire in {} seconds",
            registration_data.code(),
            ctx.registration_ttl()
        ),
    )) {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
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

    let admin_data = match AdminDao::db_select_by_email(ctx.dao().db(), data.email()).await {
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

    let token = match ctx
        .token()
        .jwt()
        .encode(admin_data.id(), &None, &JwtTokenKind::Admin)
    {
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
    if data.contains_key("token_id") && data.contains_key("collection_id") {
        let token_id = match data.get("token_id").unwrap() {
            serde_json::Value::String(collection_id) => match Uuid::from_str(&collection_id) {
                Ok(collection_id) => collection_id,
                Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
            },
            _ => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    "'collection_id' field must be of type uuid",
                )
            }
        };
        let collection_id = match data.get("collection_id").unwrap() {
            serde_json::Value::String(collection_id) => match Uuid::from_str(&collection_id) {
                Ok(collection_id) => collection_id,
                Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
            },
            _ => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    "'collection_id' field must be of type uuid",
                )
            }
        };

        let (token_data, collection_data) = match tokio::try_join!(
            TokenDao::db_select(ctx.dao().db(), &token_id),
            CollectionDao::db_select(ctx.dao().db(), &collection_id)
        ) {
            Ok(data) => data,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

        let fields = HashSet::from_iter([("_id")]);

        let mut filters = Vec::with_capacity(data.len() - 1);
        for (field_name, field_value) in data.iter() {
            if collection_data.schema_fields().contains_key(field_name) || field_name == "_id" {
                if field_name != "collection_id" {
                    let field_kind = collection_data.schema_fields().get(field_name).unwrap();
                    let column_value =
                        match ColumnValue::from_serde_json(field_kind.kind(), field_value) {
                            Ok(value) => value,
                            Err(err) => {
                                return Response::error_raw(
                                    &StatusCode::BAD_REQUEST,
                                    &format!("Error in field '{}': {}", field_name, err),
                                )
                            }
                        };
                    filters.push(RecordFilter::new(
                        &Some(field_name.to_owned()),
                        "=",
                        &Some(column_value),
                        &None,
                    ));
                }
            } else {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Field '{field_name}' is not exist in the collection"),
                );
            }
        }

        let (records_data, total) = match RecordDao::db_select_many(
            ctx.dao().db(),
            &fields,
            &collection_data,
            &RecordFilters::new(&filters),
            &Vec::new(),
            &Vec::new(),
            &RecordPagination::new(&None),
        )
        .await
        {
            Ok(data) => data,
            Err(_) => todo!(),
        };

        if total != 1 {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                "The record doesn't exist or more than one was found",
            );
        }

        let record_id = match records_data[0].data().get("_id") {
            Some(record_id) => match record_id {
                ColumnValue::Uuid(record_id) => match record_id {
                    Some(record_id) => record_id,
                    None => {
                        return Response::error_raw(
                            &StatusCode::INTERNAL_SERVER_ERROR,
                            "The '_id' field in the record is null",
                        )
                    }
                },
                _ => {
                    return Response::error_raw(
                        &StatusCode::INTERNAL_SERVER_ERROR,
                        "The '_id' field in the record is not of type uuid",
                    )
                }
            },
            None => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    "The record doesn't have '_id' field",
                )
            }
        };

        let token = match ctx.token().jwt().encode(
            token_data.id(),
            &Some(UserClaim::new(collection_data.id(), record_id)),
            &JwtTokenKind::Token,
        ) {
            Ok(token) => token,
            Err(err) => {
                return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
            }
        };

        Response::data(&StatusCode::OK, &None, &AuthTokenResJson::new(&token))
    } else {
        let token_id = match data.get("token_id") {
            Some(token_id) => match token_id {
                serde_json::Value::String(token_id) => match Uuid::from_str(&token_id) {
                    Ok(token_id) => token_id,
                    Err(err) => {
                        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string())
                    }
                },
                _ => {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        "'token_id' field must be of type uuid",
                    )
                }
            },
            None => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    "Auth without collection requires 'token_id' field of type uuid",
                )
            }
        };
        let token = match data.get("token") {
            Some(token) => match token {
                serde_json::Value::String(token) => token,
                _ => {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        "'token' field must be of type string",
                    )
                }
            },
            None => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    "Auth without collection requires 'token' field of type string",
                )
            }
        };

        let token_data = match TokenDao::db_select(ctx.dao().db(), &token_id).await {
            Ok(data) => data,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

        if token_data.token() != token {
            return Response::error_raw(&StatusCode::BAD_REQUEST, "Token doesn't match");
        }

        let token = match ctx
            .token()
            .jwt()
            .encode(token_data.id(), &None, &JwtTokenKind::Token)
        {
            Ok(token) => token,
            Err(err) => {
                return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
            }
        };

        Response::data(&StatusCode::OK, &None, &AuthTokenResJson::new(&token))
    }
}

async fn request_password_reset(
    ctx: web::Data<ApiRestCtx>,
    data: web::Json<RequestPasswordResetReqJson>,
) -> HttpResponse {
    if let Err(err) = data.validate() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
    };

    let admin_data = match AdminDao::db_select_by_email(ctx.dao().db(), data.email()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let password_reset_data = AdminPasswordResetDao::new(admin_data.id());

    if let Err(err) = password_reset_data.db_insert(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    if let Err(err)= ctx.mailer()
        .sender()
        .send(MailPayload::new(
            data.email(),
            "Request Password Reset Verification Code",
            &format!(
                "Your request password reset verification code is {}. This code will expire in {} seconds",
                password_reset_data.code(),
                ctx.reset_password_ttl()
            ),
        )) {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());

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

    if let Err(err) = ctx.mailer().sender().send(MailPayload::new(
        admin_data.email(),
        "Your Password Has Been Reset Successfully",
        "Your account password has been successfully changed",
    )) {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &ConfirmPasswordResetResJson::new(admin_data.id()),
    )
}
