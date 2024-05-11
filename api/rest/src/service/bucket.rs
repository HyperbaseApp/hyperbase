use actix_web::{http::StatusCode, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::Utc;
use hb_dao::{
    admin::AdminDao,
    bucket::BucketDao,
    change::{ChangeDao, ChangeState, ChangeTable},
    project::ProjectDao,
};
use hb_token_jwt::claim::ClaimId;

use crate::{
    context::ApiRestCtx,
    model::{
        bucket::{
            BucketResJson, DeleteBucketResJson, DeleteOneBucketReqPath, FindManyBucketReqPath,
            FindOneBucketReqPath, InsertOneBucketReqJson, InsertOneBucketReqPath,
            UpdateOneBucketReqJson, UpdateOneBucketReqPath,
        },
        PaginationRes, Response,
    },
    util,
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
    auth: BearerAuth,
    path: web::Path<InsertOneBucketReqPath>,
    data: web::Json<InsertOneBucketReqJson>,
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

    let bucket_data = match BucketDao::new(
        project_data.id(),
        data.name(),
        ctx.bucket_path(),
        data.opt_ttl(),
    )
    .await
    {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if let Err(err) = bucket_data.db_insert(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    let change_data = ChangeDao::new(
        &ChangeTable::Bucket,
        bucket_data.id(),
        &ChangeState::Upsert,
        bucket_data.updated_at(),
    );
    if let Err(err) = util::gossip_broadcast::save_change_data_and_broadcast(
        ctx.dao().db(),
        change_data,
        ctx.internal_broadcast(),
    )
    .await
    {
        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    Response::data(
        &StatusCode::CREATED,
        &None,
        &BucketResJson::new(
            bucket_data.id(),
            bucket_data.created_at(),
            bucket_data.updated_at(),
            bucket_data.project_id(),
            bucket_data.name(),
            bucket_data.opt_ttl(),
        ),
    )
}

async fn find_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<FindOneBucketReqPath>,
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

    let (project_data, bucket_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        BucketDao::db_select(ctx.dao().db(), path.bucket_id())
    ) {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != &admin_id {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if bucket_data.project_id() != project_data.id() {
        return Response::error_raw(&StatusCode::FORBIDDEN, "This bucket does not belong to you");
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &BucketResJson::new(
            bucket_data.id(),
            bucket_data.created_at(),
            bucket_data.updated_at(),
            bucket_data.project_id(),
            bucket_data.name(),
            bucket_data.opt_ttl(),
        ),
    )
}

async fn update_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<UpdateOneBucketReqPath>,
    data: web::Json<UpdateOneBucketReqJson>,
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

    let (project_data, mut bucket_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        BucketDao::db_select(ctx.dao().db(), path.bucket_id())
    ) {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != &admin_id {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if bucket_data.project_id() != project_data.id() {
        return Response::error_raw(&StatusCode::FORBIDDEN, "This bucket does not belong to you");
    }

    if let Some(name) = data.name() {
        bucket_data.set_name(name);
    }

    if let Some(opt_ttl) = data.opt_ttl() {
        bucket_data.set_opt_ttl(opt_ttl);
    }

    if !data.is_all_none() {
        if let Err(err) = bucket_data.db_update(ctx.dao().db()).await {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
        }

        let change_data = ChangeDao::new(
            &ChangeTable::Bucket,
            bucket_data.id(),
            &ChangeState::Upsert,
            bucket_data.updated_at(),
        );
        if let Err(err) = util::gossip_broadcast::save_change_data_and_broadcast(
            ctx.dao().db(),
            change_data,
            ctx.internal_broadcast(),
        )
        .await
        {
            return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
        }
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &BucketResJson::new(
            bucket_data.id(),
            bucket_data.created_at(),
            bucket_data.updated_at(),
            bucket_data.project_id(),
            bucket_data.name(),
            bucket_data.opt_ttl(),
        ),
    )
}

async fn delete_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<DeleteOneBucketReqPath>,
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

    let (project_data, bucket_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        BucketDao::db_select(ctx.dao().db(), path.bucket_id())
    ) {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != &admin_id {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if bucket_data.project_id() != project_data.id() {
        return Response::error_raw(&StatusCode::FORBIDDEN, "This bucket does not belong to you");
    }

    let deleted_at = Utc::now();

    if let Err(err) = BucketDao::db_delete(ctx.dao().db(), path.bucket_id()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    let change_data = ChangeDao::new(
        &ChangeTable::Bucket,
        bucket_data.id(),
        &ChangeState::Delete,
        &deleted_at,
    );
    if let Err(err) = util::gossip_broadcast::save_change_data_and_broadcast(
        ctx.dao().db(),
        change_data,
        ctx.internal_broadcast(),
    )
    .await
    {
        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &DeleteBucketResJson::new(bucket_data.id()),
    )
}

async fn find_many(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<FindManyBucketReqPath>,
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

    let (project_data, buckets_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        BucketDao::db_select_many_by_project_id(ctx.dao().db(), path.project_id())
    ) {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != &admin_id {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    Response::data(
        &StatusCode::OK,
        &Some(PaginationRes::new(&buckets_data.len(), &buckets_data.len())),
        &buckets_data
            .iter()
            .map(|data| {
                BucketResJson::new(
                    data.id(),
                    data.created_at(),
                    data.updated_at(),
                    data.project_id(),
                    data.name(),
                    data.opt_ttl(),
                )
            })
            .collect::<Vec<_>>(),
    )
}
