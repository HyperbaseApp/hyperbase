use actix_web::{error, web, HttpResponse, Responder, Result};
use hb_dao::{register::RegistrationDao, Db};

use crate::{
    v1::model::auth::{
        ConfirmPasswordResetJson, PasswordBasedJson, RegisterJson, RequestPasswordResetJson,
        TokenBasedJson,
    },
    Context,
};

pub fn auth_api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
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
    let password_hash = ctx
        .hash
        .argon2
        .hash_password(data.password().as_bytes())
        .map_err(|err| error::ErrorInternalServerError(err))?;

    let registration_data =
        RegistrationDao::new(data.email().to_string(), password_hash.to_string());

    registration_data
        .insert(Db::ScyllaDb(&ctx.db.scylladb))
        .await
        .map_err(|err| error::ErrorInternalServerError(err))?;

    Ok(HttpResponse::Ok().body(format!(
        "auth register {} {} {}",
        data.email(),
        data.password(),
        password_hash
    )))
}

async fn password_based(data: web::Json<PasswordBasedJson>) -> impl Responder {
    HttpResponse::Ok().body(format!(
        "auth password_based {} {}",
        data.email(),
        data.password()
    ))
}

async fn token_based(data: web::Json<TokenBasedJson>) -> impl Responder {
    HttpResponse::Ok().body(format!("auth token_based {}", data.token(),))
}

async fn request_password_reset(data: web::Json<RequestPasswordResetJson>) -> impl Responder {
    HttpResponse::Ok().body(format!("auth token_based {}", data.email(),))
}

async fn confirm_password_reset(data: web::Json<ConfirmPasswordResetJson>) -> impl Responder {
    HttpResponse::Ok().body(format!(
        "auth confirm_password_reset {} {}",
        data.code(),
        data.password()
    ))
}
