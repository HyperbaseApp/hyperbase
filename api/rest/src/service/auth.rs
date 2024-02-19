use actix_web::{http::StatusCode, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use hb_dao::{
    admin::AdminDao, admin_password_reset::AdminPasswordResetDao, register::RegistrationDao,
    token::TokenDao,
};
use hb_mailer::MailPayload;
use hb_token_jwt::kind::JwtTokenKind;
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
    todo!()
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
