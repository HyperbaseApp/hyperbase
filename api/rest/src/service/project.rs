use actix_web::{http::StatusCode, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use futures::future;
use hb_dao::{
    admin::AdminDao,
    bucket::BucketDao,
    bucket_rule::BucketRuleDao,
    collection::CollectionDao,
    collection_rule::CollectionRuleDao,
    file::FileDao,
    project::ProjectDao,
    record::{RecordDao, RecordFilters, RecordPagination},
    token::TokenDao,
    value::ColumnValue,
};
use hb_token_jwt::kind::JwtTokenKind;
use validator::Validate;

use crate::{
    context::ApiRestCtx,
    model::{
        project::{
            DeleteOneProjectReqPath, DuplicateOneProjectReqJson, DuplicateOneProjectReqPath,
            FindOneProjectReqPath, InsertOneProjectReqJson, ProjectIDResJson, ProjectResJson,
            TransferOneProjectReqJson, TransferOneProjectReqPath, UpdateOneProjectReqJson,
            UpdateOneProjectReqPath,
        },
        PaginationRes, Response,
    },
};

pub fn project_api(cfg: &mut web::ServiceConfig) {
    cfg.route("/project", web::post().to(insert_one))
        .route("/project/{project_id}", web::get().to(find_one))
        .route("/project/{project_id}", web::patch().to(update_one))
        .route("/project/{project_id}", web::delete().to(delete_one))
        .route(
            "/project/{project_id}/transfer",
            web::post().to(transfer_one),
        )
        .route(
            "/project/{project_id}/duplicate",
            web::post().to(duplicate_one),
        )
        .route("/projects", web::get().to(find_many));
}

async fn insert_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    data: web::Json<InsertOneProjectReqJson>,
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

    let project_data = ProjectDao::new(token_claim.id(), data.name());

    if let Err(err) = project_data.db_insert(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::CREATED,
        &None,
        &ProjectResJson::new(
            project_data.id(),
            project_data.created_at(),
            project_data.updated_at(),
            project_data.name(),
        ),
    )
}

async fn find_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<FindOneProjectReqPath>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let admin_id = match token_claim.kind() {
        JwtTokenKind::Admin => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => *data.id(),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::UserAnonymous | JwtTokenKind::User => {
            match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
                Ok(data) => *data.admin_id(),
                Err(err) => {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        &format!("Failed to get token data: {err}"),
                    )
                }
            }
        }
    };

    let project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if &admin_id != project_data.admin_id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &ProjectResJson::new(
            project_data.id(),
            project_data.created_at(),
            project_data.updated_at(),
            project_data.name(),
        ),
    )
}

async fn update_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<UpdateOneProjectReqPath>,
    data: web::Json<UpdateOneProjectReqJson>,
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

    let mut project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if let Some(name) = data.name() {
        project_data.set_name(name);
    }

    if !data.is_all_none() {
        if let Err(err) = project_data.db_update(ctx.dao().db()).await {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
        }
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &ProjectResJson::new(
            project_data.id(),
            project_data.created_at(),
            project_data.updated_at(),
            project_data.name(),
        ),
    )
}

async fn delete_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<DeleteOneProjectReqPath>,
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

    if let Err(err) = ProjectDao::db_delete(ctx.dao().db(), path.project_id()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &ProjectIDResJson::new(project_data.id()),
    )
}

async fn transfer_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<TransferOneProjectReqPath>,
    data: web::Json<TransferOneProjectReqJson>,
) -> HttpResponse {
    if let Err(err) = data.validate() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
    }

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

    let mut project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    let admin_data = match AdminDao::db_select_by_email(ctx.dao().db(), data.admin_email()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    project_data.set_admin_id(admin_data.id());

    let mut tokens_data =
        match TokenDao::db_select_many_by_project_id(ctx.dao().db(), path.project_id()).await {
            Ok(data) => data,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

    let mut update_token_fut = Vec::with_capacity(tokens_data.len());
    for token_data in &mut tokens_data {
        token_data.set_admin_id(admin_data.id());
        token_data.set_name(&format!("[{}] {}", project_data.name(), token_data.name()));
        update_token_fut.push(token_data.db_update(ctx.dao().db()));
    }

    if let Err(err) = tokio::try_join!(
        future::try_join_all(update_token_fut),
        project_data.db_update(ctx.dao().db())
    ) {
        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &ProjectIDResJson::new(project_data.id()),
    )
}

async fn duplicate_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<DuplicateOneProjectReqPath>,
    data: web::Json<DuplicateOneProjectReqJson>,
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

    let new_project_data = ProjectDao::new(
        project_data.admin_id(),
        &format!("[DUPLICATE] {}", project_data.name()),
    );
    if let Err(err) = new_project_data.db_insert(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    let (collections_data, buckets_data, tokens_data) = match tokio::try_join!(
        CollectionDao::db_select_many_by_project_id(ctx.dao().db(), project_data.id(),),
        BucketDao::db_select_many_by_project_id(ctx.dao().db(), project_data.id()),
        TokenDao::db_select_many_by_project_id(ctx.dao().db(), project_data.id())
    ) {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let mut old_new_collection_id_map = HashMap::with_capacity(collections_data.len());
    for collection_data in &collections_data {
        let new_collection_data = CollectionDao::new(
            new_project_data.id(),
            collection_data.name(),
            collection_data.schema_fields(),
            collection_data.opt_auth_column_id(),
        );
        if let Err(err) = new_collection_data.db_insert(ctx.dao().db()).await {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
        }

        old_new_collection_id_map.insert(collection_data.id(), *new_collection_data.id());

        if *data.with_records() {
            let (records_data, _) = match RecordDao::db_select_many(
                ctx.dao().db(),
                &HashSet::new(),
                collection_data,
                &None,
                &RecordFilters::new(&Vec::new()),
                &Vec::new(),
                &Vec::new(),
                &RecordPagination::new(&None),
            )
            .await
            {
                Ok(data) => data,
                Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
            };

            for record_data in &records_data {
                let created_by = match record_data.get("_created_by") {
                    Some(data) => match data {
                        ColumnValue::Uuid(uuid) => match uuid {
                            Some(uuid) => uuid,
                            None => {
                                return Response::error_raw(
                                    &StatusCode::INTERNAL_SERVER_ERROR,
                                    &format!("Field '_created_by' can't be null"),
                                )
                            }
                        },
                        _ => {
                            return Response::error_raw(
                                &StatusCode::INTERNAL_SERVER_ERROR,
                                &format!("Field '_created_by' isn't of type UUID"),
                            )
                        }
                    },
                    None => {
                        return Response::error_raw(
                            &StatusCode::INTERNAL_SERVER_ERROR,
                            &format!("Field '_created_by' isn't found in the record"),
                        )
                    }
                };
                let new_record_data = RecordDao::new(
                    created_by,
                    new_collection_data.id(),
                    &Some(record_data.len()),
                );
                if let Err(err) = new_record_data.db_insert(ctx.dao().db()).await {
                    return Response::error_raw(
                        &StatusCode::INTERNAL_SERVER_ERROR,
                        &err.to_string(),
                    );
                }
            }
        }
    }

    let mut old_new_bucket_id_map = HashMap::with_capacity(buckets_data.len());
    for bucket_data in &buckets_data {
        let new_bucket_data = match BucketDao::new(
            new_project_data.id(),
            bucket_data.name(),
            ctx.bucket_path(),
        )
        .await
        {
            Ok(data) => data,
            Err(err) => {
                return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
            }
        };
        if let Err(err) = new_bucket_data.db_insert(ctx.dao().db()).await {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
        }

        old_new_bucket_id_map.insert(bucket_data.id(), *new_bucket_data.id());

        if *data.with_files() {
            let (files_data, _) = match FileDao::db_select_many_by_bucket_id(
                ctx.dao().db(),
                bucket_data.id(),
                &None,
                &None,
            )
            .await
            {
                Ok(data) => data,
                Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
            };

            for file_data in &files_data {
                let new_file_data = FileDao::new(
                    file_data.created_by(),
                    new_bucket_data.id(),
                    file_data.file_name(),
                    file_data.content_type(),
                    file_data.size(),
                );
                if let Err(err) = new_file_data
                    .save(
                        ctx.dao().db(),
                        new_bucket_data.path(),
                        &format!("{}/{}", bucket_data.path(), file_data.id()),
                    )
                    .await
                {
                    return Response::error_raw(
                        &StatusCode::INTERNAL_SERVER_ERROR,
                        &err.to_string(),
                    );
                }
            }
        }
    }

    for token_data in &tokens_data {
        let new_token_data = TokenDao::new(
            new_project_data.id(),
            token_data.admin_id(),
            token_data.name(),
            ctx.access_token_length(),
            token_data.allow_anonymous(),
            token_data.expired_at(),
        );
        if let Err(err) = new_token_data.db_insert(ctx.dao().db()).await {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
        }

        let (collection_rules_data, bucket_rules_data) = match tokio::try_join!(
            CollectionRuleDao::db_select_many_by_token_id(ctx.dao().db(), token_data.id()),
            BucketRuleDao::db_select_many_by_token_id(ctx.dao().db(), token_data.id())
        ) {
            Ok(data) => data,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

        for collection_rule_data in &collection_rules_data {
            let new_collection_id =
                match old_new_collection_id_map.get(collection_rule_data.collection_id()) {
                    Some(id) => id,
                    None => {
                        return Response::error_raw(
                            &StatusCode::INTERNAL_SERVER_ERROR,
                            &format!(
                                "Collection id '{}' doesn't found in this project",
                                collection_rule_data.collection_id()
                            ),
                        )
                    }
                };

            let new_collection_rule_data = CollectionRuleDao::new(
                new_project_data.id(),
                new_token_data.id(),
                new_collection_id,
                collection_rule_data.find_one(),
                collection_rule_data.find_many(),
                collection_rule_data.insert_one(),
                collection_rule_data.update_one(),
                collection_rule_data.delete_one(),
            );
            if let Err(err) = new_collection_rule_data.db_insert(ctx.dao().db()).await {
                return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
            }
        }

        for bucket_rule_data in &bucket_rules_data {
            let new_bucket_id = match old_new_bucket_id_map.get(bucket_rule_data.bucket_id()) {
                Some(id) => id,
                None => {
                    return Response::error_raw(
                        &StatusCode::INTERNAL_SERVER_ERROR,
                        &format!(
                            "Bucket id '{}' doesn't found in this project",
                            bucket_rule_data.bucket_id()
                        ),
                    )
                }
            };

            let new_bucket_rule_data = BucketRuleDao::new(
                new_project_data.id(),
                new_token_data.id(),
                new_bucket_id,
                bucket_rule_data.find_one(),
                bucket_rule_data.find_many(),
                bucket_rule_data.insert_one(),
                bucket_rule_data.update_one(),
                bucket_rule_data.delete_one(),
            );
            if let Err(err) = new_bucket_rule_data.db_insert(ctx.dao().db()).await {
                return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
            }
        }
    }

    Response::data(
        &StatusCode::CREATED,
        &None,
        &ProjectResJson::new(
            new_project_data.id(),
            new_project_data.created_at(),
            new_project_data.updated_at(),
            new_project_data.name(),
        ),
    )
}

async fn find_many(ctx: web::Data<ApiRestCtx>, auth: BearerAuth) -> HttpResponse {
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

    let projects_data =
        match ProjectDao::db_select_many_by_admin_id(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => data,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

    Response::data(
        &StatusCode::OK,
        &Some(PaginationRes::new(
            &projects_data.len(),
            &projects_data.len(),
        )),
        &projects_data
            .iter()
            .map(|data| {
                ProjectResJson::new(data.id(), data.created_at(), data.updated_at(), data.name())
            })
            .collect::<Vec<_>>(),
    )
}
