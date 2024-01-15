use actix_web::{http::StatusCode, web, HttpResponse};
use hb_dao::{admin::AdminDao, project::ProjectDao};
use hb_token_jwt::kind::JwtTokenKind;

use crate::{
    context::ApiRestCtx,
    model::{
        bucket::{
            DeleteOneBucketReqPath, FindManyBucketReqPath, FindOneBucketReqPath,
            InsertOneBucketReqJson, InsertOneBucketReqPath, UpdateOneBucketReqJson,
            UpdateOneBucketReqPath,
        },
        Response, TokenReqHeader,
    },
};

pub fn bucket_api(cfg: &mut web::ServiceConfig) {
    cfg.route("/project/{project_id}/bucket", web::post().to(insert_one))
        .route(
            "/project/{project_id}/bucket/{bucket_id}",
            web::get().to(find_one),
        )
        .route(
            "/project/{project_id}/bucket/{bucket_id}",
            web::patch().to(update_one),
        )
        .route(
            "/project/{project_id}/bucket/{bucket_id}",
            web::delete().to(delete_one),
        )
        .route("/project/{project_id}/buckets", web::get().to(find_many));
}

async fn insert_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<InsertOneBucketReqPath>,
    data: web::Json<InsertOneBucketReqJson>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error_raw(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    if let Err(err) = AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            &format!("Failed to get user data: {err}"),
        );
    }

    let project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    // todo!()

    Response::data(&StatusCode::CREATED, &None, "todo!()")
}

async fn find_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<FindOneBucketReqPath>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error_raw(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    if let Err(err) = AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            &format!("Failed to get user data: {err}"),
        );
    }

    // todo!()

    Response::data(&StatusCode::OK, &None, "todo!()")
}

async fn update_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<UpdateOneBucketReqPath>,
    data: web::Json<UpdateOneBucketReqJson>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error_raw(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    if let Err(err) = AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            &format!("Failed to get user data: {err}"),
        );
    }

    // todo!()

    Response::data(&StatusCode::OK, &None, "todo!()")
}

async fn delete_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<DeleteOneBucketReqPath>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error_raw(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    if let Err(err) = AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            &format!("Failed to get user data: {err}"),
        );
    }

    // todo!()

    Response::data(&StatusCode::OK, &None, "todo!()")
}

async fn find_many(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<FindManyBucketReqPath>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error_raw(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    if let Err(err) = AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            &format!("Failed to get user data: {err}"),
        );
    }

    let project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    // todo!()

    Response::data(&StatusCode::OK, &None, "todo!()")
}
