use actix_web::{http::StatusCode, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::{Duration, Utc};
use hb_dao::{admin::AdminDao, project::ProjectDao, token::TokenDao};
use hb_token_jwt::claim::ClaimId;

use crate::{
    context::ApiRestCtx,
    model::{
        token::{
            DeleteOneTokenReqPath, DeleteTokenResJson, FindManyTokenReqPath, FindOneTokenReqPath,
            InsertOneTokenReqJson, InsertOneTokenReqPath, TokenResJson, UpdateOneTokenReqJson,
            UpdateOneTokenReqPath,
        },
        PaginationRes, Response,
    },
};

pub fn token_api(cfg: &mut web::ServiceConfig) {
    cfg.route("/project/{project_id}/token", web::post().to(insert_one))
        .route(
            "/project/{project_id}/token/{token_id}",
            web::get().to(find_one),
        )
        .route(
            "/project/{project_id}/token/{token_id}",
            web::patch().to(update_one),
        )
        .route(
            "/project/{project_id}/token/{token_id}",
            web::delete().to(delete_one),
        )
        .route("/project/{project_id}/tokens", web::get().to(find_many));
}

async fn insert_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<InsertOneTokenReqPath>,
    data: web::Json<InsertOneTokenReqJson>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let admin_id = match token_claim.id() {
        ClaimId::Admin(id) => match AdminDao::db_select(ctx.dao().db(), id).await {
            Ok(data) => *data.id(),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::UNAUTHORIZED,
                    &format!("Failed to get admin data: {err}"),
                )
            }
        },
        ClaimId::Token(_, _) => {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                "Must be logged in using password-based login",
            )
        }
    };

    let project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != &admin_id {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if let Some(expired_at) = data.expired_at() {
        if (*expired_at - Utc::now()) < Duration::zero() {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                "Expiration date must be in the future",
            );
        }
    }

    let token_data = TokenDao::new(
        project_data.id(),
        &admin_id,
        data.name(),
        ctx.access_token_length(),
        data.allow_anonymous(),
        data.expired_at(),
    );
    if let Err(err) = token_data.db_insert(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::CREATED,
        &None,
        &TokenResJson::new(
            token_data.id(),
            token_data.created_at(),
            token_data.updated_at(),
            token_data.project_id(),
            token_data.name(),
            token_data.token(),
            token_data.allow_anonymous(),
            token_data.expired_at(),
        ),
    )
}

async fn find_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<FindOneTokenReqPath>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let admin_id = match token_claim.id() {
        ClaimId::Admin(id) => match AdminDao::db_select(ctx.dao().db(), id).await {
            Ok(data) => *data.id(),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::UNAUTHORIZED,
                    &format!("Failed to get admin data: {err}"),
                )
            }
        },
        ClaimId::Token(_, _) => {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                "Must be logged in using password-based login",
            )
        }
    };

    let project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != &admin_id {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    let token_data = match TokenDao::db_select(ctx.dao().db(), path.token_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_data.admin_id() != &admin_id {
        return Response::error_raw(&StatusCode::FORBIDDEN, "This token does not belong to you");
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &TokenResJson::new(
            token_data.id(),
            token_data.created_at(),
            token_data.updated_at(),
            token_data.project_id(),
            token_data.name(),
            token_data.token(),
            token_data.allow_anonymous(),
            token_data.expired_at(),
        ),
    )
}

async fn update_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<UpdateOneTokenReqPath>,
    data: web::Json<UpdateOneTokenReqJson>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let admin_id = match token_claim.id() {
        ClaimId::Admin(id) => match AdminDao::db_select(ctx.dao().db(), id).await {
            Ok(data) => *data.id(),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::UNAUTHORIZED,
                    &format!("Failed to get admin data: {err}"),
                )
            }
        },
        ClaimId::Token(_, _) => {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                "Must be logged in using password-based login",
            )
        }
    };

    let project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != &admin_id {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    let mut token_data = match TokenDao::db_select(ctx.dao().db(), path.token_id()).await {
        Ok(data) => data,
        Err(err) => {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                &format!("Failed to get token data: {err}"),
            )
        }
    };

    if token_data.admin_id() != &admin_id {
        return Response::error_raw(&StatusCode::FORBIDDEN, "This token does not belong to you");
    }

    if let Some(name) = data.name() {
        token_data.set_name(name);
    }

    if let Some(allow_anonymous) = data.allow_anonymous() {
        token_data.set_allow_anonymous(allow_anonymous);
    }

    if let Some(expired_at) = data.expired_at() {
        if let Some(expired_at) = expired_at {
            if (*expired_at - Utc::now()) < Duration::zero() {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    "Expiration date must be in the future",
                );
            }
        }

        token_data.set_expired_at(expired_at);
    }

    if !data.is_all_none() {
        if let Err(err) = token_data.db_update(ctx.dao().db()).await {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
        }
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &TokenResJson::new(
            token_data.id(),
            token_data.created_at(),
            token_data.updated_at(),
            token_data.project_id(),
            token_data.name(),
            token_data.token(),
            token_data.allow_anonymous(),
            token_data.expired_at(),
        ),
    )
}

async fn delete_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<DeleteOneTokenReqPath>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let admin_id = match token_claim.id() {
        ClaimId::Admin(id) => match AdminDao::db_select(ctx.dao().db(), id).await {
            Ok(data) => *data.id(),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::UNAUTHORIZED,
                    &format!("Failed to get admin data: {err}"),
                )
            }
        },
        ClaimId::Token(_, _) => {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                "Must be logged in using password-based login",
            )
        }
    };

    let project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != &admin_id {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    let token_data = match TokenDao::db_select(ctx.dao().db(), path.token_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_data.admin_id() != &admin_id {
        return Response::error_raw(&StatusCode::FORBIDDEN, "This token does not belong to you");
    }

    if let Err(err) = TokenDao::db_delete(ctx.dao().db(), path.token_id()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &DeleteTokenResJson::new(token_data.id()),
    )
}

async fn find_many(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<FindManyTokenReqPath>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let admin_id = match token_claim.id() {
        ClaimId::Admin(id) => match AdminDao::db_select(ctx.dao().db(), id).await {
            Ok(data) => *data.id(),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::UNAUTHORIZED,
                    &format!("Failed to get admin data: {err}"),
                )
            }
        },
        ClaimId::Token(_, _) => {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                "Must be logged in using password-based login",
            )
        }
    };

    let project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != &admin_id {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    let tokens_data = match TokenDao::db_select_many_by_admin_id_and_project_id(
        ctx.dao().db(),
        &admin_id,
        path.project_id(),
    )
    .await
    {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let mut tokens_res = Vec::with_capacity(tokens_data.len());
    for token_data in &tokens_data {
        tokens_res.push(TokenResJson::new(
            token_data.id(),
            token_data.created_at(),
            token_data.updated_at(),
            token_data.project_id(),
            token_data.name(),
            token_data.token(),
            token_data.allow_anonymous(),
            token_data.expired_at(),
        ));
    }

    Response::data(
        &StatusCode::OK,
        &Some(PaginationRes::new(&tokens_data.len(), &tokens_data.len())),
        &tokens_res,
    )
}
