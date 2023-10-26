use actix_web::{http::StatusCode, web, HttpResponse};
use hb_dao::{
    admin::AdminDao, admin_password_reset::AdminPasswordResetDao, register::RegistrationDao,
    token::TokenDao, Db,
};
use hb_mailer::MailPayload;
use hb_token_jwt::kind::JwtTokenKind;
use validator::Validate;

use crate::{
    v1::model::{
        auth::{
            ConfirmPasswordResetReqJson, ConfirmPasswordResetResJson, PasswordBasedReqJson,
            PasswordBasedResJson, RegisterReqJson, RegisterResJson, RequestPasswordResetReqJson,
            RequestPasswordResetResJson, TokenBasedReqJson, TokenBasedResJson,
            VerifyRegistrationReqJson, VerifyRegistrationResJson,
        },
        Response,
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

async fn register(ctx: web::Data<Context>, data: web::Json<RegisterReqJson>) -> HttpResponse {
    if let Err(err) = data.validate() {
        return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str());
    }

    let scylladb = Db::ScyllaDb(&ctx.db.scylladb);

    if let Ok(_) = AdminDao::select_by_email(&scylladb, data.email()).await {
        return Response::error(StatusCode::BAD_REQUEST, "Account has been registered");
    };

    let password_hash = match ctx.hash.argon2.hash_password(data.password().as_bytes()) {
        Ok(hash) => hash,
        Err(err) => {
            return Response::error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string().as_str())
        }
    };

    let registration_data = RegistrationDao::new(data.email(), &password_hash.to_string());

    if let Err(err) = registration_data.insert(&scylladb).await {
        return Response::error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string().as_str());
    }

    if let Err(err) = ctx.mailer.sender.send(MailPayload::new(
        data.email(),
        "Registration Verification Code",
        &format!(
            "Your registration verification code is {}. This code will expire in {} seconds",
            registration_data.code(),
            ctx.verify_code_ttl
        ),
    )) {
        return Response::error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string().as_str());
    }

    Response::data(
        StatusCode::OK,
        None,
        RegisterResJson::new(registration_data.id()),
    )
}

async fn verify_registration(
    ctx: web::Data<Context>,
    data: web::Json<VerifyRegistrationReqJson>,
) -> HttpResponse {
    let scylladb = Db::ScyllaDb(&ctx.db.scylladb);

    let registration_data = match RegistrationDao::select(&scylladb, data.id()).await {
        Ok(data) => data,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str()),
    };

    if data.code() != registration_data.code() {
        return Response::error(StatusCode::BAD_REQUEST, "Wrong code");
    }

    let admin_data = AdminDao::new(registration_data.email(), registration_data.password_hash());

    if let Err(err) = admin_data.insert(&scylladb).await {
        return Response::error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string().as_str());
    }

    if let Err(err) = registration_data.delete(&scylladb).await {
        return Response::error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string().as_str());
    }

    Response::data(
        StatusCode::CREATED,
        None,
        VerifyRegistrationResJson::new(admin_data.id()),
    )
}

async fn password_based(
    ctx: web::Data<Context>,
    data: web::Json<PasswordBasedReqJson>,
) -> HttpResponse {
    if let Err(err) = data.validate() {
        return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str());
    }

    let scylladb = Db::ScyllaDb(&ctx.db.scylladb);

    let admin_data = match AdminDao::select_by_email(&scylladb, data.email()).await {
        Ok(data) => data,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str()),
    };

    if let Err(err) = ctx
        .hash
        .argon2
        .verify_password(data.password(), admin_data.password_hash())
    {
        return Response::error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string().as_str());
    }

    let token = match ctx.token.jwt.encode(admin_data.id(), &JwtTokenKind::Admin) {
        Ok(token) => token,
        Err(err) => {
            return Response::error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string().as_str())
        }
    };

    Response::data(StatusCode::OK, None, PasswordBasedResJson::new(&token))
}

async fn token_based(ctx: web::Data<Context>, data: web::Json<TokenBasedReqJson>) -> HttpResponse {
    let scylladb = Db::ScyllaDb(&ctx.db.scylladb);

    let token_data = match TokenDao::select_by_token(&scylladb, data.token()).await {
        Ok(data) => data,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str()),
    };

    let token = match ctx.token.jwt.encode(token_data.id(), &JwtTokenKind::Token) {
        Ok(token) => token,
        Err(err) => {
            return Response::error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string().as_str())
        }
    };

    Response::data(StatusCode::OK, None, TokenBasedResJson::new(&token))
}

async fn request_password_reset(
    ctx: web::Data<Context>,
    data: web::Json<RequestPasswordResetReqJson>,
) -> HttpResponse {
    if let Err(err) = data.validate() {
        return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str());
    };

    let scylladb = Db::ScyllaDb(&ctx.db.scylladb);

    let admin_data = match AdminDao::select_by_email(&scylladb, data.email()).await {
        Ok(data) => data,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str()),
    };

    let password_reset_data = AdminPasswordResetDao::new(admin_data.id());

    if let Err(err) = password_reset_data.insert(&scylladb).await {
        return Response::error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string().as_str());
    }

    if let Err(err)= ctx.mailer
        .sender
        .send(MailPayload::new(
            data.email(),
            "Request Password Reset Verification Code",
            &format!(
                "Your request password reset verification code is {}. This code will expire in {} seconds",
                password_reset_data.code(),
                ctx.verify_code_ttl
            ),
        )) {
            return Response::error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string().as_str());

        }

    Response::data(
        StatusCode::OK,
        None,
        RequestPasswordResetResJson::new(password_reset_data.id()),
    )
}

async fn confirm_password_reset(
    ctx: web::Data<Context>,
    data: web::Json<ConfirmPasswordResetReqJson>,
) -> HttpResponse {
    let scylladb = Db::ScyllaDb(&ctx.db.scylladb);

    let password_reset_data = match AdminPasswordResetDao::select(&scylladb, data.id()).await {
        Ok(data) => data,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str()),
    };

    if data.code() != password_reset_data.code() {
        return Response::error(StatusCode::BAD_REQUEST, "Wrong code");
    }

    let mut admin_data = match AdminDao::select(&scylladb, password_reset_data.admin_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str()),
    };

    let password_hash = match ctx.hash.argon2.hash_password(data.password().as_bytes()) {
        Ok(hash) => hash,
        Err(err) => {
            return Response::error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string().as_str())
        }
    };

    admin_data.set_password_hash(&password_hash.to_string());

    if let Err(err) = admin_data.update(&scylladb).await {
        return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str());
    }

    if let Err(err) = ctx.mailer.sender.send(MailPayload::new(
        admin_data.email(),
        "Your Password Has Been Reset Successfully",
        "Your account password has been successfully changed",
    )) {
        return Response::error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string().as_str());
    }

    Response::data(
        StatusCode::OK,
        None,
        ConfirmPasswordResetResJson::new(admin_data.id()),
    )
}
