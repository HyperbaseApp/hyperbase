use actix_web::{http::StatusCode, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use chrono::{Duration, Utc};
use futures::future;
use hb_dao::{
    admin::AdminDao, bucket::BucketDao, collection::CollectionDao, project::ProjectDao,
    record::RecordDao, token::TokenDao,
};
use hb_token_jwt::kind::JwtTokenKind;

use crate::{
    context::ApiRestCtx,
    model::{
        token::{
            DeleteOneTokenReqPath, DeleteOneTokenReqQuery, DeleteTokenResJson,
            FindManyTokenReqQuery, FindOneTokenReqPath, FindOneTokenReqQuery,
            InsertOneTokenReqJson, TokenBucketRuleMethodJson, TokenCollectionRuleMethodJson,
            TokenResJson, UpdateOneTokenReqJson, UpdateOneTokenReqPath, UpdateOneTokenReqQuery,
        },
        PaginationRes, Response,
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
    auth: BearerAuth,
    data: web::Json<InsertOneTokenReqJson>,
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

    let project_data = match ProjectDao::db_select(ctx.dao().db(), data.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
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

    let mut data_bucket_rules = HashMap::new();
    if let Some(bucket_rules) = data.bucket_rules() {
        let mut buckets_data_fut = Vec::with_capacity(bucket_rules.len());
        for bucket_id in bucket_rules.keys() {
            buckets_data_fut.push(BucketDao::db_select(ctx.dao().db(), bucket_id));
        }
        if let Err(err) = future::try_join_all(buckets_data_fut).await {
            return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
        }
        data_bucket_rules = HashMap::with_capacity(bucket_rules.len());
        for (bucket_id, bucket_rules) in bucket_rules {
            let bucket_rules = match bucket_rules.to_dao() {
                Ok(bucket_rules) => bucket_rules,
                Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
            };
            data_bucket_rules.insert(*bucket_id, bucket_rules);
        }
    }

    let mut data_collection_rules = HashMap::new();
    if let Some(collection_rules) = data.collection_rules() {
        let mut collections_data_fut = Vec::with_capacity(collection_rules.len());
        let mut check_tables_must_exist_fut = Vec::with_capacity(collection_rules.len());
        for collection_id in collection_rules.keys() {
            collections_data_fut.push(CollectionDao::db_select(ctx.dao().db(), collection_id));
            check_tables_must_exist_fut.push(RecordDao::db_check_table_must_exist(
                ctx.dao().db(),
                collection_id,
            ));
        }
        let mut project_ids = HashSet::with_capacity(collections_data_fut.len());
        match future::try_join_all(collections_data_fut).await {
            Ok(collections_data) => {
                for collection_data in collections_data {
                    project_ids.insert(*collection_data.project_id());
                }
            }
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        }
        let mut projects_data_fut = Vec::with_capacity(project_ids.len());
        for project_id in &project_ids {
            projects_data_fut.push(ProjectDao::db_select(ctx.dao().db(), project_id));
        }
        match future::try_join_all(projects_data_fut).await {
            Ok(projects_data) => {
                for project_data in projects_data {
                    if project_data.admin_id() != token_claim.id() {
                        return Response::error_raw(
                            &StatusCode::FORBIDDEN,
                            "This collection does not belong to you",
                        );
                    }
                }
            }
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        }
        if let Err(err) = future::try_join_all(check_tables_must_exist_fut).await {
            return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
        }

        data_collection_rules = HashMap::with_capacity(collection_rules.len());
        for (collection_id, collection_rules) in collection_rules {
            let collection_rules = match collection_rules.to_dao() {
                Ok(collection_rules) => collection_rules,
                Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
            };
            data_collection_rules.insert(*collection_id, collection_rules);
        }
    }

    let token_data: TokenDao = TokenDao::new(
        project_data.id(),
        token_claim.id(),
        ctx.access_token_length(),
        &data_bucket_rules,
        &data_collection_rules,
        data.expired_at(),
    );
    if let Err(err) = token_data.db_insert(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    let mut token_data_collection_rules =
        HashMap::with_capacity(token_data.collection_rules().len());
    for (collection_id, collection_rules) in token_data.collection_rules() {
        let collection_rules = match TokenCollectionRuleMethodJson::from_dao(collection_rules) {
            Ok(collection_rules) => collection_rules,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };
        token_data_collection_rules.insert(*collection_id, collection_rules);
    }

    let mut token_data_bucket_rules = HashMap::with_capacity(token_data.bucket_rules().len());
    for (bucket_id, bucket_rules) in token_data.bucket_rules() {
        let bucket_rules = match TokenBucketRuleMethodJson::from_dao(bucket_rules) {
            Ok(bucket_rules) => bucket_rules,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };
        token_data_bucket_rules.insert(*bucket_id, bucket_rules);
    }

    Response::data(
        &StatusCode::CREATED,
        &None,
        &TokenResJson::new(
            token_data.id(),
            token_data.created_at(),
            token_data.updated_at(),
            token_data.token(),
            &token_data_bucket_rules,
            &token_data_collection_rules,
            token_data.expired_at(),
        ),
    )
}

async fn find_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<FindOneTokenReqPath>,
    query: web::Query<FindOneTokenReqQuery>,
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

    let project_data = match ProjectDao::db_select(ctx.dao().db(), query.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    let token_data = match TokenDao::db_select(ctx.dao().db(), path.token_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_data.admin_id() != token_claim.id() {
        return Response::error_raw(&StatusCode::FORBIDDEN, "This token does not belong to you");
    }

    let mut token_data_bucket_rules = HashMap::with_capacity(token_data.bucket_rules().len());
    for (bucket_id, bucket_rules) in token_data.bucket_rules() {
        let bucket_rules = match TokenBucketRuleMethodJson::from_dao(bucket_rules) {
            Ok(bucket_rules) => bucket_rules,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };
        token_data_bucket_rules.insert(*bucket_id, bucket_rules);
    }

    let mut token_data_collection_rules =
        HashMap::with_capacity(token_data.collection_rules().len());
    for (collection_id, collection_rules) in token_data.collection_rules() {
        let collection_rules = match TokenCollectionRuleMethodJson::from_dao(collection_rules) {
            Ok(collection_rules) => collection_rules,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };
        token_data_collection_rules.insert(*collection_id, collection_rules);
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &TokenResJson::new(
            token_data.id(),
            token_data.created_at(),
            token_data.updated_at(),
            token_data.token(),
            &token_data_bucket_rules,
            &token_data_collection_rules,
            token_data.expired_at(),
        ),
    )
}

async fn update_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<UpdateOneTokenReqPath>,
    query: web::Query<UpdateOneTokenReqQuery>,
    data: web::Json<UpdateOneTokenReqJson>,
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

    let project_data = match ProjectDao::db_select(ctx.dao().db(), query.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
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
                &format!("Failed to get token data: {}", err.to_string()),
            )
        }
    };

    if token_data.admin_id() != token_claim.id() {
        return Response::error_raw(&StatusCode::FORBIDDEN, "This token does not belong to you");
    }

    if let Some(collection_rules) = data.collection_rules() {
        let mut collections_data_fut = Vec::with_capacity(collection_rules.len());
        let mut check_tables_must_exist_fut = Vec::with_capacity(collection_rules.len());
        for collection_id in collection_rules.keys() {
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
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get collection: {}", err.to_string()),
                )
            }
        }
        let mut projects_data_fut = Vec::with_capacity(project_ids.len());
        for project_id in &project_ids {
            projects_data_fut.push(ProjectDao::db_select(ctx.dao().db(), project_id));
        }
        match future::try_join_all(projects_data_fut).await {
            Ok(projects_data) => {
                for project_data in projects_data {
                    if project_data.admin_id() != token_claim.id() {
                        return Response::error_raw(
                            &StatusCode::FORBIDDEN,
                            "This collection does not belong to you",
                        );
                    }
                }
            }
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        }
        if let Err(err) = future::try_join_all(check_tables_must_exist_fut).await {
            return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
        }

        let mut data_rules = HashMap::with_capacity(collection_rules.len());
        for (collection_id, collection_rules) in collection_rules {
            let collection_rules = match collection_rules.to_dao() {
                Ok(collection_rules) => collection_rules,
                Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
            };
            data_rules.insert(*collection_id, collection_rules);
        }

        token_data.set_collection_rules(&data_rules);
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

    let mut token_data_bucket_rules = HashMap::with_capacity(token_data.bucket_rules().len());
    for (bucket_id, bucket_rules) in token_data.bucket_rules() {
        let bucket_rules = match TokenBucketRuleMethodJson::from_dao(bucket_rules) {
            Ok(bucket_rules) => bucket_rules,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };
        token_data_bucket_rules.insert(*bucket_id, bucket_rules);
    }

    let mut token_data_collection_rules =
        HashMap::with_capacity(token_data.collection_rules().len());
    for (collection_id, collection_rules) in token_data.collection_rules() {
        let collection_rules = match TokenCollectionRuleMethodJson::from_dao(collection_rules) {
            Ok(collection_rules) => collection_rules,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };
        token_data_collection_rules.insert(*collection_id, collection_rules);
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &TokenResJson::new(
            token_data.id(),
            token_data.created_at(),
            token_data.updated_at(),
            token_data.token(),
            &token_data_bucket_rules,
            &token_data_collection_rules,
            token_data.expired_at(),
        ),
    )
}

async fn delete_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<DeleteOneTokenReqPath>,
    query: web::Query<DeleteOneTokenReqQuery>,
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

    let project_data = match ProjectDao::db_select(ctx.dao().db(), query.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    let token_data = match TokenDao::db_select(ctx.dao().db(), path.token_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_data.admin_id() != token_claim.id() {
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
    query: web::Query<FindManyTokenReqQuery>,
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

    let project_data = match ProjectDao::db_select(ctx.dao().db(), query.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    let tokens_data =
        match TokenDao::db_select_many_by_admin_id(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => data,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

    let mut tokens_res = Vec::with_capacity(tokens_data.len());
    for token_data in &tokens_data {
        let mut token_data_bucket_rules = HashMap::with_capacity(token_data.bucket_rules().len());
        for (bucket_id, bucket_rules) in token_data.bucket_rules() {
            let bucket_rules = match TokenBucketRuleMethodJson::from_dao(bucket_rules) {
                Ok(bucket_rules) => bucket_rules,
                Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
            };
            token_data_bucket_rules.insert(*bucket_id, bucket_rules);
        }

        let mut token_data_collection_rules =
            HashMap::with_capacity(token_data.collection_rules().len());
        for (collection_id, collection_rules) in token_data.collection_rules() {
            let collection_rules = match TokenCollectionRuleMethodJson::from_dao(collection_rules) {
                Ok(collection_rules) => collection_rules,
                Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
            };
            token_data_collection_rules.insert(*collection_id, collection_rules);
        }

        tokens_res.push(TokenResJson::new(
            token_data.id(),
            token_data.created_at(),
            token_data.updated_at(),
            token_data.token(),
            &token_data_bucket_rules,
            &token_data_collection_rules,
            token_data.expired_at(),
        ));
    }

    Response::data(
        &StatusCode::OK,
        &Some(PaginationRes::new(&tokens_data.len(), &tokens_data.len())),
        &tokens_res,
    )
}
