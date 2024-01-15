use actix_web::{http::StatusCode, web, HttpResponse};
use hb_dao::{admin::AdminDao, token::TokenDao};
use hb_token_jwt::kind::JwtTokenKind;

use crate::{
    context::ApiRestCtx,
    model::{
        file::{
            DeleteOneFileReqPath, FindManyFileReqPath, FindOneFileReqPath, InsertOneFileReqPath,
            UpdateOneFileReqPath,
        },
        Response, TokenReqHeader,
    },
};

pub fn file_api(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/project/{project_id}/bucket/{bucket_id}/file",
        web::post().to(insert_one),
    )
    .route(
        "/project/{project_id}/bucket/{bucket_id}/file/{file_id}",
        web::get().to(find_one),
    )
    .route(
        "/project/{project_id}/bucket/{bucket_id}/file/{file_id}",
        web::patch().to(update_one),
    )
    .route(
        "/project/{project_id}/bucket/{bucket_id}/file/{file_id}",
        web::delete().to(delete_one),
    )
    .route(
        "/project/{project_id}/bucket/{bucket_id}/files",
        web::post().to(find_many),
    );
}

async fn insert_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<InsertOneFileReqPath>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error_raw(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.admin_id(), Some(data)),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get token data: {err}"),
                )
            }
        },
    };

    if let Some(token_data) = &token_data {
        if !token_data.is_allow_insert_file(path.bucket_id()) {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to write data to this bucket",
            );
        }
    }

    // todo!()

    Response::data(&StatusCode::CREATED, &None, "todo!()")
}

async fn find_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<FindOneFileReqPath>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error_raw(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.admin_id(), Some(data)),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get token data: {err}"),
                )
            }
        },
    };

    if let Some(token_data) = &token_data {
        if !token_data.is_allow_find_one_file(path.bucket_id()) {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to read this bucket",
            );
        }
    }

    // todo!()

    Response::data(&StatusCode::OK, &None, "todo!()")
}

async fn update_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<UpdateOneFileReqPath>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error_raw(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.admin_id(), Some(data)),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get token data: {err}"),
                )
            }
        },
    };

    if let Some(token_data) = &token_data {
        if !token_data.is_allow_update_file(path.bucket_id()) {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to update this file",
            );
        }
    }

    // todo!()

    Response::data(&StatusCode::OK, &None, "todo!()")
}

async fn delete_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<DeleteOneFileReqPath>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error_raw(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.admin_id(), Some(data)),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get token data: {err}"),
                )
            }
        },
    };

    if let Some(token_data) = &token_data {
        if !token_data.is_allow_delete_file(path.bucket_id()) {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to delete this file",
            );
        }
    }

    // todo!()

    Response::data(&StatusCode::OK, &None, "todo!()")
}

async fn find_many(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<FindManyFileReqPath>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error_raw(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.admin_id(), Some(data)),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get token data: {err}"),
                )
            }
        },
    };

    if let Some(token_data) = &token_data {
        if !token_data.is_allow_find_many_files(path.bucket_id()) {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to read these files",
            );
        }
    }

    // todo!()

    Response::data(&StatusCode::OK, &None, "todo!()")
}
