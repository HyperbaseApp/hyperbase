use actix_web::{http::StatusCode, web, HttpResponse};
use hb_token_jwt::kind::JwtTokenKind;

use crate::{
    context::Context,
    model::{Response, TokenReqHeader},
};

pub fn token_api(cfg: &mut web::ServiceConfig) {
    cfg.route("/admin/token", web::post().to(insert_one))
        .route("/admin/token/{token_id}", web::get().to(find_one))
        .route("/admin/token/{token_id}", web::patch().to(update_one))
        .route("/admin/token/{token_id}", web::delete().to(delete_one))
        .route("/admin/tokens", web::get().to(find_many));
}

async fn insert_one(ctx: web::Data<Context>, token: web::Header<TokenReqHeader>) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token.jwt.decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error(
            StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    Response::data(StatusCode::CREATED, None, "TODO!")
}

async fn find_one(ctx: web::Data<Context>, token: web::Header<TokenReqHeader>) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token.jwt.decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error(
            StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    Response::data(StatusCode::OK, None, "TODO!")
}

async fn update_one(ctx: web::Data<Context>, token: web::Header<TokenReqHeader>) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token.jwt.decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error(
            StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    Response::data(StatusCode::OK, None, "TODO!")
}

async fn delete_one(ctx: web::Data<Context>, token: web::Header<TokenReqHeader>) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token.jwt.decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error(
            StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    Response::data(StatusCode::OK, None, "TODO!")
}

async fn find_many(ctx: web::Data<Context>, token: web::Header<TokenReqHeader>) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token.jwt.decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error(
            StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    Response::data(StatusCode::OK, None, "TODO!")
}
