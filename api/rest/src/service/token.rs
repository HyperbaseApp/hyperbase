use actix_web::{http::StatusCode, web, HttpResponse};
use ahash::{HashSet, HashSetExt};
use chrono::{Duration, Utc};
use futures::future;
use hb_dao::{
    admin::AdminDao, collection::CollectionDao, project::ProjectDao, record::RecordDao,
    token::TokenDao,
};
use hb_token_jwt::kind::JwtTokenKind;

use crate::{
    context::ApiRestCtx,
    model::{
        token::{
            DeleteOneTokenReqPath, DeleteTokenResJson, FindOneTokenReqPath, InsertOneTokenReqJson,
            TokenResJson, UpdateOneTokenReqJson, UpdateOneTokenReqPath,
        },
        PaginationRes, Response, TokenReqHeader,
    },
};

pub fn token_api(cfg: &mut web::ServiceConfig) {
    cfg.route("/admin/token", web::post().to(insert_one))
        .route("/admin/token/{token_id}", web::get().to(find_one))
        .route("/admin/token/{token_id}", web::put().to(update_one))
        .route("/admin/token/{token_id}", web::delete().to(delete_one))
        .route("/admin/tokens", web::get().to(find_many));
}

async fn insert_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    data: web::Json<InsertOneTokenReqJson>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    if let Err(err) = AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
        return Response::error(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    if data.rules().is_empty() {
        return Response::error(&StatusCode::BAD_REQUEST, "Rules can't be empty");
    }

    if let Some(expired_at) = data.expired_at() {
        if (*expired_at - Utc::now()) < Duration::zero() {
            return Response::error(
                &StatusCode::BAD_REQUEST,
                "Expiration date must be in the future",
            );
        }
    }

    let mut collections_data_fut = Vec::with_capacity(data.rules().len());
    let mut check_tables_must_exist_fut = Vec::with_capacity(data.rules().len());
    for collection_id in data.rules().keys() {
        collections_data_fut.push(CollectionDao::db_select(ctx.dao().db(), collection_id));
        check_tables_must_exist_fut.push(RecordDao::db_check_table_must_exist(
            ctx.dao().db(),
            collection_id,
        ));
    }
    let mut project_ids = HashSet::new();
    match future::try_join_all(collections_data_fut).await {
        Ok(collections_data) => {
            for collection_data in collections_data {
                project_ids.insert(*collection_data.project_id());
            }
        }
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    }
    let mut projects_data_fut = Vec::with_capacity(project_ids.len());
    for project_id in &project_ids {
        projects_data_fut.push(ProjectDao::db_select(ctx.dao().db(), project_id));
    }
    match future::try_join_all(projects_data_fut).await {
        Ok(projects_data) => {
            for project_data in projects_data {
                if project_data.admin_id() != token_claim.id() {
                    return Response::error(
                        &StatusCode::FORBIDDEN,
                        "This collection does not belong to you",
                    );
                }
            }
        }
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    }
    if let Err(err) = future::try_join_all(check_tables_must_exist_fut).await {
        return Response::error(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    let token_data = TokenDao::new(
        token_claim.id(),
        ctx.access_token_length(),
        data.rules(),
        data.expired_at(),
    );
    if let Err(err) = token_data.db_insert(ctx.dao().db()).await {
        return Response::error(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::CREATED,
        &None,
        &TokenResJson::new(
            token_data.id(),
            token_data.created_at(),
            token_data.updated_at(),
            token_data.token(),
            token_data.rules(),
            token_data.expired_at(),
        ),
    )
}

async fn find_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<FindOneTokenReqPath>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    if let Err(err) = AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
        return Response::error(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    let token_data = match TokenDao::db_select(ctx.dao().db(), path.token_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_data.admin_id() != token_claim.id() {
        return Response::error(&StatusCode::FORBIDDEN, "This token does not belong to you");
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &TokenResJson::new(
            token_data.id(),
            token_data.created_at(),
            token_data.updated_at(),
            token_data.token(),
            token_data.rules(),
            token_data.expired_at(),
        ),
    )
}

async fn update_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<UpdateOneTokenReqPath>,
    data: web::Json<UpdateOneTokenReqJson>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    if let Err(err) = AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
        return Response::error(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    let mut token_data = match TokenDao::db_select(ctx.dao().db(), path.token_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_data.admin_id() != token_claim.id() {
        return Response::error(&StatusCode::FORBIDDEN, "This token does not belong to you");
    }

    if let Some(expired_at) = data.expired_at() {
        if (*expired_at - Utc::now()) < Duration::zero() {
            return Response::error(
                &StatusCode::BAD_REQUEST,
                "Expiration date must be in the future",
            );
        }
    }

    if let Some(rules) = data.rules() {
        let mut collections_data_fut = Vec::with_capacity(rules.len());
        let mut check_tables_must_exist_fut = Vec::with_capacity(rules.len());
        for collection_id in rules.keys() {
            collections_data_fut.push(CollectionDao::db_select(ctx.dao().db(), collection_id));
            check_tables_must_exist_fut.push(RecordDao::db_check_table_must_exist(
                ctx.dao().db(),
                collection_id,
            ));
        }
        let mut project_ids = HashSet::new();
        match future::try_join_all(collections_data_fut).await {
            Ok(collections_data) => {
                for collection_data in collections_data {
                    project_ids.insert(*collection_data.project_id());
                }
            }
            Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
        }
        let mut projects_data_fut = Vec::with_capacity(project_ids.len());
        for project_id in &project_ids {
            projects_data_fut.push(ProjectDao::db_select(ctx.dao().db(), project_id));
        }
        match future::try_join_all(projects_data_fut).await {
            Ok(projects_data) => {
                for project_data in projects_data {
                    if project_data.admin_id() != token_claim.id() {
                        return Response::error(
                            &StatusCode::FORBIDDEN,
                            "This collection does not belong to you",
                        );
                    }
                }
            }
            Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
        }
        if let Err(err) = future::try_join_all(check_tables_must_exist_fut).await {
            return Response::error(&StatusCode::BAD_REQUEST, &err.to_string());
        }
    }

    if let Some(rules) = data.rules() {
        token_data.set_rules(rules);
    }
    token_data.set_expired_at(data.expired_at());
    if let Err(err) = token_data.db_update(ctx.dao().db()).await {
        return Response::error(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &TokenResJson::new(
            token_data.id(),
            token_data.created_at(),
            token_data.updated_at(),
            token_data.token(),
            token_data.rules(),
            token_data.expired_at(),
        ),
    )
}

async fn delete_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<DeleteOneTokenReqPath>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    if let Err(err) = AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
        return Response::error(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    let token_data = match TokenDao::db_select(ctx.dao().db(), path.token_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_data.admin_id() != token_claim.id() {
        return Response::error(&StatusCode::FORBIDDEN, "This token does not belong to you");
    }

    if let Err(err) = TokenDao::db_delete(ctx.dao().db(), path.token_id()).await {
        return Response::error(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &DeleteTokenResJson::new(token_data.id()),
    )
}

async fn find_many(ctx: web::Data<ApiRestCtx>, token: web::Header<TokenReqHeader>) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    if let Err(err) = AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
        return Response::error(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    let tokens_data =
        match TokenDao::db_select_many_by_admin_id(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => data,
            Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

    Response::data(
        &StatusCode::OK,
        &Some(PaginationRes::new(&tokens_data.len(), &tokens_data.len())),
        &tokens_data
            .iter()
            .map(|data| {
                TokenResJson::new(
                    data.id(),
                    data.created_at(),
                    data.updated_at(),
                    data.token(),
                    data.rules(),
                    data.expired_at(),
                )
            })
            .collect::<Vec<_>>(),
    )
}
