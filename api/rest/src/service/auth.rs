use actix_web::{http::StatusCode, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use ahash::{HashSet, HashSetExt};
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
        JwtTokenKind::User => {
            if let Err(err) = TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get token data: {err}"),
                );
            }
        }
        _ => todo!(),
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

    if let Err(err) = ctx
        .mailer()
        .sender()
        .send(MailPayload::new(
            data.email(),
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

        for auth_column in collection_data.auth_columns() {
            if !data.data().as_ref().unwrap().contains_key(auth_column) {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    "Incorrect authentication data",
                );
            }
        }

        let mut record_fields = HashSet::with_capacity(data.data().as_ref().unwrap().len() + 1);
        record_fields.insert("_id");
        let mut record_filter_childs = Vec::with_capacity(data.data().as_ref().unwrap().len());
        for (field, value) in data.data().as_ref().unwrap() {
            if collection_data.auth_columns().contains(field) {
                record_fields.insert(field.as_str());
                let schema_field = match collection_data.schema_fields().get(field) {
                    Some(schema_field) => schema_field,
                    None => {
                        return Response::error_raw(
                            &StatusCode::BAD_REQUEST,
                            &format!("Field {field} doesn't exist in the collection"),
                        )
                    }
                };
                let column_value = match ColumnValue::from_serde_json(schema_field.kind(), &value) {
                    Ok(column_value) => column_value,
                    Err(err) => {
                        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string())
                    }
                };
                record_filter_childs.push(RecordFilter::new(
                    &Some(field.to_owned()),
                    "=",
                    &Some(column_value),
                    &None,
                ));
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
        )
        .await
        {
            Ok(data) => data,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

        if total == 0 {
            return Response::error_raw(&StatusCode::BAD_REQUEST, "Bad authentication data. Please ask admin to review auth columns in this collection.");
        } else if total > 1 {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                "Multiple authentication data is found",
            );
        }

        let record_id = if let ColumnValue::Uuid(Some(id)) = records_data[0].get("_id").unwrap() {
            id
        } else {
            return Response::error_raw(
                &StatusCode::INTERNAL_SERVER_ERROR,
                "Can't parse id of authentication data",
            );
        };

        match ctx.token().jwt().encode(
            token_data.id(),
            &Some(UserClaim::new(collection_id, record_id)),
            &JwtTokenKind::User,
        ) {
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
            .encode(token_data.id(), &None, &JwtTokenKind::UserAnonymous)
        {
            Ok(token) => token,
            Err(err) => {
                return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
            }
        }
    };

    Response::data(&StatusCode::OK, &None, &AuthTokenResJson::new(&token))
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
        )).await {
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

    if let Err(err) = ctx
        .mailer()
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

    Response::data(
        &StatusCode::OK,
        &None,
        &ConfirmPasswordResetResJson::new(admin_data.id()),
    )
}
