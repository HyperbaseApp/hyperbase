use actix_web::{http::StatusCode, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use hb_dao::{
    admin::AdminDao, collection::CollectionDao, collection_rule::CollectionRuleDao,
    project::ProjectDao, token::TokenDao,
};
use hb_token_jwt::kind::JwtTokenKind;

use crate::{
    context::ApiRestCtx,
    model::{
        collection_rule::{
            CollectionRuleResJson, DeleteCollectionRuleResJson, DeleteOneCollectionRuleReqPath,
            FindManyCollectionRuleReqPath, FindOneCollectionRuleReqPath,
            InsertOneCollectionRuleReqJson, InsertOneCollectionRuleReqPath,
            UpdateOneCollectionRuleReqJson, UpdateOneCollectionRuleReqPath,
        },
        PaginationRes, Response,
    },
};

pub fn collection_rule_api(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/project/{project_id}/token/{token_id}/collection_rule",
        web::post().to(insert_one),
    )
    .route(
        "/project/{project_id}/token/{token_id}/collection_rule/{rule_id}",
        web::get().to(find_one),
    )
    .route(
        "/project/{project_id}/token/{token_id}/collection_rule/{rule_id}",
        web::patch().to(update_one),
    )
    .route(
        "/project/{project_id}/token/{token_id}/collection_rule/{rule_id}",
        web::delete().to(delete_one),
    )
    .route(
        "/project/{project_id}/token/{token_id}/collection_rules",
        web::get().to(find_many),
    );
}

async fn insert_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<InsertOneCollectionRuleReqPath>,
    data: web::Json<InsertOneCollectionRuleReqJson>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::Admin {
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

    let (project_data, token_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        TokenDao::db_select(ctx.dao().db(), path.token_id()),
        CollectionDao::db_select(ctx.dao().db(), data.collection_id())
    ) {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if token_data.project_id() != project_data.id() {
        return Response::error_raw(&StatusCode::FORBIDDEN, "This token does not belong to you");
    }

    if collection_data.project_id() != project_data.id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This collection does not belong to you",
        );
    }

    let collection_rule_data = CollectionRuleDao::new(
        project_data.id(),
        token_data.id(),
        collection_data.id(),
        data.find_one(),
        data.find_many(),
        data.insert_one(),
        data.update_one(),
        data.delete_one(),
    );

    if let Err(err) = collection_rule_data.db_insert(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::CREATED,
        &None,
        &CollectionRuleResJson::new(
            collection_rule_data.id(),
            collection_rule_data.created_at(),
            collection_rule_data.updated_at(),
            collection_rule_data.project_id(),
            collection_rule_data.token_id(),
            collection_rule_data.collection_id(),
            collection_rule_data.find_one(),
            collection_rule_data.find_many(),
            collection_rule_data.insert_one(),
            collection_rule_data.update_one(),
            collection_rule_data.delete_one(),
        ),
    )
}

async fn find_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<FindOneCollectionRuleReqPath>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::Admin {
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

    let (token_data, collection_rule_data) = match tokio::try_join!(
        TokenDao::db_select(ctx.dao().db(), path.token_id()),
        CollectionRuleDao::db_select(ctx.dao().db(), path.rule_id()),
    ) {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_data.admin_id() != token_claim.id() {
        return Response::error_raw(&StatusCode::FORBIDDEN, "This token does not belong to you");
    }

    if collection_rule_data.token_id() != token_data.id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This collection rule does not belong to you",
        );
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &CollectionRuleResJson::new(
            collection_rule_data.id(),
            collection_rule_data.created_at(),
            collection_rule_data.updated_at(),
            collection_rule_data.project_id(),
            collection_rule_data.token_id(),
            collection_rule_data.collection_id(),
            collection_rule_data.find_one(),
            collection_rule_data.find_many(),
            collection_rule_data.insert_one(),
            collection_rule_data.update_one(),
            collection_rule_data.delete_one(),
        ),
    )
}

async fn update_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<UpdateOneCollectionRuleReqPath>,
    data: web::Json<UpdateOneCollectionRuleReqJson>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::Admin {
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

    let (token_data, mut collection_rule_data) = match tokio::try_join!(
        TokenDao::db_select(ctx.dao().db(), path.token_id()),
        CollectionRuleDao::db_select(ctx.dao().db(), path.rule_id()),
    ) {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_data.admin_id() != token_claim.id() {
        return Response::error_raw(&StatusCode::FORBIDDEN, "This token does not belong to you");
    }

    if collection_rule_data.token_id() != token_data.id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This collection rule does not belong to you",
        );
    }

    if let Some(find_one) = data.find_one() {
        collection_rule_data.set_find_one(find_one);
    }

    if let Some(find_many) = data.find_many() {
        collection_rule_data.set_find_many(find_many);
    }

    if let Some(insert_one) = data.insert_one() {
        collection_rule_data.set_insert_one(insert_one);
    }

    if let Some(update_one) = data.update_one() {
        collection_rule_data.set_update_one(update_one);
    }

    if let Some(delete_one) = data.delete_one() {
        collection_rule_data.set_delete_one(delete_one);
    }

    if !data.is_all_none() {
        if let Err(err) = collection_rule_data.db_update(ctx.dao().db()).await {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
        }
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &CollectionRuleResJson::new(
            collection_rule_data.id(),
            collection_rule_data.created_at(),
            collection_rule_data.updated_at(),
            collection_rule_data.project_id(),
            collection_rule_data.token_id(),
            collection_rule_data.collection_id(),
            collection_rule_data.find_one(),
            collection_rule_data.find_many(),
            collection_rule_data.insert_one(),
            collection_rule_data.update_one(),
            collection_rule_data.delete_one(),
        ),
    )
}

async fn delete_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<DeleteOneCollectionRuleReqPath>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::Admin {
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

    let (token_data, collection_rule_data) = match tokio::try_join!(
        TokenDao::db_select(ctx.dao().db(), path.token_id()),
        CollectionRuleDao::db_select(ctx.dao().db(), path.rule_id()),
    ) {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_data.admin_id() != token_claim.id() {
        return Response::error_raw(&StatusCode::FORBIDDEN, "This token does not belong to you");
    }

    if collection_rule_data.token_id() != token_data.id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This collection rule does not belong to you",
        );
    }

    if let Err(err) = CollectionRuleDao::db_delete(ctx.dao().db(), path.rule_id()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &DeleteCollectionRuleResJson::new(collection_rule_data.id()),
    )
}

async fn find_many(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<FindManyCollectionRuleReqPath>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::Admin {
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

    let (token_data, collection_rules_data) = match tokio::try_join!(
        TokenDao::db_select(ctx.dao().db(), path.token_id()),
        CollectionRuleDao::db_select_many_by_token_id(ctx.dao().db(), path.token_id(),),
    ) {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_data.admin_id() != token_claim.id() {
        return Response::error_raw(&StatusCode::FORBIDDEN, "This token does not belong to you");
    }

    Response::data(
        &StatusCode::OK,
        &Some(PaginationRes::new(
            &collection_rules_data.len(),
            &collection_rules_data.len(),
        )),
        &collection_rules_data
            .iter()
            .map(|data| {
                CollectionRuleResJson::new(
                    data.id(),
                    data.created_at(),
                    data.updated_at(),
                    data.project_id(),
                    data.token_id(),
                    data.collection_id(),
                    data.find_one(),
                    data.find_many(),
                    data.insert_one(),
                    data.update_one(),
                    data.delete_one(),
                )
            })
            .collect::<Vec<_>>(),
    )
}
