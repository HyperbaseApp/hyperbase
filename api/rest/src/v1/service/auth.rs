use actix_web::{error, web, HttpResponse, Responder, Result};
use hb_dao::{
    admin::AdminDao, admin_password_reset::AdminPasswordResetDao, register::RegistrationDao,
    token::TokenDao, Db,
};
use hb_mailer::MailPayload;
use hb_token_jwt::kind::JwtTokenKind;
use validator::Validate;

use crate::{
    v1::model::auth::{
        ConfirmPasswordResetJson, PasswordBasedJson, RegisterJson, RequestPasswordResetJson,
        TokenBasedJson, VerifyRegistrationJson,
    },
    Context,
};

pub fn auth_api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/verify-registration", web::post().to(verify_registration))
            .route("/password-based", web::post().to(password_based))
            .route("/token-based", web::post().to(token_based))
            .route(
                "/request-password-reset",
                web::post().to(request_password_reset),
            )
            .route(
                "/confirm-password-reset",
                web::post().to(confirm_password_reset),
            ),
    );
}

async fn register(
    ctx: web::Data<Context>,
    data: web::Json<RegisterJson>,
) -> Result<impl Responder> {
    data.validate().map_err(|err| error::ErrorBadRequest(err))?;

    let scylladb = Db::ScyllaDb(&ctx.db.scylladb);

    let password_hash = ctx
        .hash
        .argon2
        .hash_password(data.password().as_bytes())
        .map_err(|err| error::ErrorInternalServerError(err))?;

    let registration_data = RegistrationDao::new(data.email(), &password_hash.to_string());

    registration_data
        .insert(&scylladb)
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?;

    ctx.mailer
        .sender
        .send(MailPayload::new(
            data.email(),
            "Registration Verification Code",
            &format!(
                "Your registration verification code is {}. This code will expire in {}",
                registration_data.code(),
                ctx.verify_code_ttl
            ),
        ))
        .map_err(|err| error::ErrorInternalServerError(err))?;

    Ok(HttpResponse::Ok().body(format!(
        "auth register {} {} {}",
        data.email(),
        data.password(),
        password_hash
    )))
}

async fn verify_registration(
    ctx: web::Data<Context>,
    data: web::Json<VerifyRegistrationJson>,
) -> Result<impl Responder> {
    let scylladb = Db::ScyllaDb(&ctx.db.scylladb);

    let registration_data = RegistrationDao::select(&scylladb, data.id())
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?;

    if data.code() != registration_data.code() {
        return Err(error::ErrorBadRequest("wrong code"));
    }

    let admin_data = AdminDao::new(registration_data.email(), registration_data.password_hash());

    admin_data
        .insert(&scylladb)
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?;

    registration_data
        .delete(&scylladb)
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?;

    Ok(HttpResponse::Ok().body(format!(
        "auth verify_registration {} {}",
        data.id(),
        data.code()
    )))
}

async fn password_based(
    ctx: web::Data<Context>,
    data: web::Json<PasswordBasedJson>,
) -> Result<impl Responder> {
    data.validate().map_err(|err| error::ErrorBadRequest(err))?;

    let scylladb = Db::ScyllaDb(&ctx.db.scylladb);

    let admin_data = AdminDao::select_by_email(&scylladb, data.email())
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?;

    ctx.hash
        .argon2
        .verify_password(data.password(), admin_data.password_hash())
        .map_err(|err| error::ErrorBadRequest(err))?;

    let token = ctx
        .token
        .jwt
        .encode(admin_data.id(), &JwtTokenKind::Admin)
        .map_err(|err| error::ErrorInternalServerError(err))?;

    Ok(HttpResponse::Ok().body(format!(
        "auth password_based {} {} {}",
        data.email(),
        data.password(),
        token
    )))
}

async fn token_based(
    ctx: web::Data<Context>,
    data: web::Json<TokenBasedJson>,
) -> Result<impl Responder> {
    let scylladb = Db::ScyllaDb(&ctx.db.scylladb);

    let token_data = TokenDao::select_by_token(&scylladb, data.token())
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?;

    let token = ctx
        .token
        .jwt
        .encode(token_data.id(), &JwtTokenKind::Token)
        .map_err(|err| error::ErrorInternalServerError(err))?;

    Ok(HttpResponse::Ok().body(format!("auth token_based {} {}", data.token(), token)))
}

async fn request_password_reset(
    ctx: web::Data<Context>,
    data: web::Json<RequestPasswordResetJson>,
) -> Result<impl Responder> {
    data.validate().map_err(|err| error::ErrorBadRequest(err))?;

    let scylladb = Db::ScyllaDb(&ctx.db.scylladb);

    let admin_data = AdminDao::select_by_email(&scylladb, data.email())
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?;

    let password_reset_data = AdminPasswordResetDao::new(admin_data.id());

    password_reset_data
        .insert(&scylladb)
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?;

    ctx.mailer
        .sender
        .send(MailPayload::new(
            data.email(),
            "Request Password Reset Verification Code",
            &format!(
                "Your request password reset verification code is {}. This code will expire in {}",
                password_reset_data.code(),
                ctx.verify_code_ttl
            ),
        ))
        .map_err(|err| error::ErrorInternalServerError(err))?;

    Ok(HttpResponse::Ok().body(format!("auth token_based {}", data.email(),)))
}

async fn confirm_password_reset(
    ctx: web::Data<Context>,
    data: web::Json<ConfirmPasswordResetJson>,
) -> Result<impl Responder> {
    let scylladb = Db::ScyllaDb(&ctx.db.scylladb);

    let password_reset_data = AdminPasswordResetDao::select(&scylladb, data.id())
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?;

    if data.code() != password_reset_data.code() {
        return Err(error::ErrorBadRequest("wrong code"));
    }

    let mut admin_data = AdminDao::select(&scylladb, password_reset_data.admin_id())
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?;

    let password_hash = ctx
        .hash
        .argon2
        .hash_password(data.password().as_bytes())
        .map_err(|err| error::ErrorInternalServerError(err))?;

    admin_data.set_password_hash(&password_hash.to_string());

    admin_data
        .update(&scylladb)
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?;

    ctx.mailer
        .sender
        .send(MailPayload::new(
            admin_data.email(),
            "Your Password Has Been Reset Successfully",
            "Your account password has been successfully changed",
        ))
        .map_err(|err| error::ErrorInternalServerError(err))?;

    Ok(HttpResponse::Ok().body(format!(
        "auth confirm_password_reset {} {}",
        data.code(),
        data.password()
    )))
}
