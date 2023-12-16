use actix_web::{http::StatusCode, web, HttpResponse};
use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use hb_dao::{
    admin::AdminDao,
    collection::{CollectionDao, SchemaFieldKind, SchemaFieldPropsModel},
    project::ProjectDao,
};
use hb_token_jwt::kind::JwtTokenKind;

use crate::{
    context::ApiRestCtx,
    model::{
        collection::{
            CollectionResJson, DeleteCollectionResJson, DeleteOneCollectionReqPath,
            FindManyCollectionReqPath, FindOneCollectionReqPath, InsertOneCollectionReqJson,
            InsertOneCollectionReqPath, SchemaFieldPropsModelJson, UpdateOneCollectionReqJson,
            UpdateOneCollectionReqPath,
        },
        PaginationRes, Response, TokenReqHeader,
    },
};

pub fn collection_api(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/project/{project_id}/collection",
        web::post().to(insert_one),
    )
    .route(
        "/project/{project_id}/collection/{collection_id}",
        web::get().to(find_one),
    )
    .route(
        "/project/{project_id}/collection/{collection_id}",
        web::patch().to(update_one),
    )
    .route(
        "/project/{project_id}/collection/{collection_id}",
        web::delete().to(delete_one),
    )
    .route(
        "/project/{project_id}/collections",
        web::get().to(find_many),
    );
}

async fn insert_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<InsertOneCollectionReqPath>,
    data: web::Json<InsertOneCollectionReqJson>,
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

    let project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    let mut schema_fields = HashMap::with_capacity(data.schema_fields().len());
    for (key, value) in data.schema_fields().iter() {
        if key.starts_with("_") || !key.chars().all(|c| c == '_' || ('a'..='z').contains(&c)) {
            return Response::error(
                &StatusCode::BAD_REQUEST,
                &format!("Field '{key}' should only have lowercase English letters and an optional underscore (_) after the first character"),
            );
        }
        if let Some(indexes) = data.indexes() {
            if let Some(field_name) = indexes.get(key) {
                if !value.required().unwrap_or(false) {
                    return Response::error(
                        &StatusCode::BAD_REQUEST,
                        &format!(
                            "Field '{field_name}' must be required because it is in the indexes"
                        ),
                    );
                }
            }
        }
        schema_fields.insert(
            key.to_string(),
            SchemaFieldPropsModel::new(
                &match SchemaFieldKind::from_str(value.kind()) {
                    Ok(kind) => kind,
                    Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
                },
                &value.required().unwrap_or(false),
            ),
        );
    }

    if let Some(indexes) = data.indexes() {
        for index in indexes {
            match schema_fields.get(index) {
                Some(field) => {
                    if !field.required() {
                        return Response::error(
                            &StatusCode::BAD_REQUEST,
                            &format!(
                                "Field '{index}' must be required because it is in the indexes"
                            ),
                        );
                    }
                }
                None => {
                    return Response::error(
                        &StatusCode::BAD_REQUEST,
                        &format!("Field '{index}' is not exist in the schema fields"),
                    )
                }
            }
        }
    }

    let collection_data = match CollectionDao::new(
        path.project_id(),
        data.name(),
        &schema_fields,
        &match data.indexes() {
            Some(indexes) => indexes.clone(),
            None => HashSet::new(),
        },
    ) {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string()),
    };
    if let Err(err) = collection_data.db_insert(ctx.dao().db()).await {
        return Response::error(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::CREATED,
        &None,
        &CollectionResJson::new(
            collection_data.id(),
            collection_data.created_at(),
            collection_data.updated_at(),
            collection_data.project_id(),
            collection_data.name(),
            &collection_data
                .schema_fields()
                .iter()
                .map(|(key, value)| {
                    (
                        key.to_owned(),
                        SchemaFieldPropsModelJson::new(
                            value.kind().to_str(),
                            &Some(*value.required()),
                        ),
                    )
                })
                .collect(),
            collection_data.indexes(),
        ),
    )
}

async fn find_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<FindOneCollectionReqPath>,
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

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        CollectionDao::db_select(ctx.dao().db(), path.collection_id()),
    ) {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if project_data.id() != collection_data.project_id() {
        return Response::error(&StatusCode::BAD_REQUEST, "Project ID does not match");
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &CollectionResJson::new(
            collection_data.id(),
            collection_data.created_at(),
            collection_data.updated_at(),
            collection_data.project_id(),
            collection_data.name(),
            &collection_data
                .schema_fields()
                .iter()
                .map(|(key, value)| {
                    (
                        key.to_owned(),
                        SchemaFieldPropsModelJson::new(
                            value.kind().to_str(),
                            &Some(*value.required()),
                        ),
                    )
                })
                .collect(),
            collection_data.indexes(),
        ),
    )
}

async fn update_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<UpdateOneCollectionReqPath>,
    data: web::Json<UpdateOneCollectionReqJson>,
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

    let (project_data, mut collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        CollectionDao::db_select(ctx.dao().db(), path.collection_id()),
    ) {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if project_data.id() != collection_data.project_id() {
        return Response::error(&StatusCode::BAD_REQUEST, "Project ID does not match");
    }

    if let Some(name) = data.name() {
        collection_data.set_name(name);
    }

    if let Some(schema_field) = data.schema_fields() {
        let mut schema_fields = HashMap::with_capacity(schema_field.len());
        for (key, value) in schema_field.iter() {
            if key.starts_with("_") || !key.chars().all(|c| c == '_' || ('a'..='z').contains(&c)) {
                return Response::error(
                    &StatusCode::BAD_REQUEST,
                    &format!("Field '{key}' should only have lowercase English letters and an optional underscore (_) after the first character"),
                );
            }
            if let Some(indexes) = data.indexes() {
                if let Some(field_name) = indexes.get(key) {
                    if !value.required().unwrap_or(false) {
                        return Response::error(
                            &StatusCode::BAD_REQUEST,
                            &format!(
                                "Field '{field_name}' must be required because it is in the indexes"
                            ),
                        );
                    }
                }
            }
            schema_fields.insert(
                key.to_owned(),
                SchemaFieldPropsModel::new(
                    &match SchemaFieldKind::from_str(value.kind()) {
                        Ok(kind) => kind,
                        Err(err) => {
                            return Response::error(
                                &StatusCode::INTERNAL_SERVER_ERROR,
                                &err.to_string(),
                            )
                        }
                    },
                    &value.required().unwrap_or(false),
                ),
            );
        }
        collection_data.update_schema_fields(&schema_fields);
    }

    if let Some(indexes) = data.indexes() {
        for index in indexes {
            match collection_data.schema_fields().get(index) {
                Some(field) => {
                    if !field.required() {
                        return Response::error(
                            &StatusCode::BAD_REQUEST,
                            &format!(
                                "Field '{index}' must be required because it is in the indexes"
                            ),
                        );
                    }
                }
                None => {
                    return Response::error(
                        &StatusCode::BAD_REQUEST,
                        &format!(
                            "Field '{index}' is in indexes but not exist in the schema fields"
                        ),
                    )
                }
            }
        }
        collection_data.update_indexes(indexes);
    }

    if !data.is_all_none() {
        if let Err(err) = collection_data.db_update(ctx.dao().db()).await {
            return Response::error(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
        }
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &CollectionResJson::new(
            collection_data.id(),
            collection_data.created_at(),
            collection_data.updated_at(),
            collection_data.project_id(),
            collection_data.name(),
            &collection_data
                .schema_fields()
                .iter()
                .map(|(key, value)| {
                    (
                        key.to_owned(),
                        SchemaFieldPropsModelJson::new(
                            value.kind().to_str(),
                            &Some(*value.required()),
                        ),
                    )
                })
                .collect(),
            collection_data.indexes(),
        ),
    )
}

async fn delete_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<DeleteOneCollectionReqPath>,
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

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        CollectionDao::db_select(ctx.dao().db(), path.collection_id()),
    ) {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if project_data.id() != collection_data.project_id() {
        return Response::error(&StatusCode::BAD_REQUEST, "Project ID does not match");
    }

    if let Err(err) = CollectionDao::db_delete(ctx.dao().db(), path.collection_id()).await {
        return Response::error(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &DeleteCollectionResJson::new(collection_data.id()),
    )
}

async fn find_many(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<FindManyCollectionReqPath>,
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

    let project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    let collections_data = match CollectionDao::db_select_many_by_project_id(
        ctx.dao().db(),
        path.project_id(),
    )
    .await
    {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    Response::data(
        &StatusCode::OK,
        &Some(PaginationRes::new(
            &collections_data.len(),
            &collections_data.len(),
        )),
        &collections_data
            .iter()
            .map(|data| {
                CollectionResJson::new(
                    data.id(),
                    data.created_at(),
                    data.updated_at(),
                    data.project_id(),
                    data.name(),
                    &data
                        .schema_fields()
                        .iter()
                        .map(|(key, value)| {
                            (
                                key.to_owned(),
                                SchemaFieldPropsModelJson::new(
                                    value.kind().to_str(),
                                    &Some(*value.required()),
                                ),
                            )
                        })
                        .collect(),
                    data.indexes(),
                )
            })
            .collect::<Vec<_>>(),
    )
}
