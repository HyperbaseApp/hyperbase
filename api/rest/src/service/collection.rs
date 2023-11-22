use std::{collections::HashMap, str::FromStr};

use actix_web::{http::StatusCode, web, HttpResponse};
use hb_dao::{
    collection::{CollectionDao, SchemaFieldKind, SchemaFieldModel},
    project::ProjectDao,
};
use hb_token_jwt::kind::JwtTokenKind;

use crate::{
    context::Context,
    model::{
        collection::{
            CollectionResJson, DeleteCollectionResJson, DeleteOneCollectionReqPath,
            FindManyCollectionReqPath, FindOneCollectionReqPath, InsertOneCollectionReqJson,
            InsertOneCollectionReqPath, SchemaFieldModelJson, UpdateOneCollectionReqJson,
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
    ctx: web::Data<Context>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<InsertOneCollectionReqPath>,
    data: web::Json<InsertOneCollectionReqJson>,
) -> HttpResponse {
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

    if let Err(err) = ProjectDao::db_select(&ctx.dao.db, path.project_id()).await {
        return Response::error(StatusCode::BAD_REQUEST, &err.to_string());
    };

    let mut schema_fields = HashMap::with_capacity(data.schema_fields().len());
    for (key, value) in data.schema_fields().iter() {
        schema_fields.insert(
            key.to_owned(),
            SchemaFieldModel::new(
                &match SchemaFieldKind::from_str(value.kind()) {
                    Ok(kind) => kind,
                    Err(err) => return Response::error(StatusCode::BAD_REQUEST, &err.to_string()),
                },
                value.required(),
            ),
        );
    }

    let collection_data = CollectionDao::new(
        path.project_id(),
        data.name(),
        &schema_fields,
        &match data.indexes() {
            Some(indexes) => indexes.to_vec(),
            None => Vec::new(),
        },
    );
    if let Err(err) = collection_data.db_insert(&ctx.dao.db).await {
        return Response::error(StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        StatusCode::CREATED,
        None,
        CollectionResJson::new(
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
                        SchemaFieldModelJson::new(&value.kind().to_string(), value.required()),
                    )
                })
                .collect(),
            collection_data.indexes(),
        ),
    )
}

async fn find_one(
    ctx: web::Data<Context>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<FindOneCollectionReqPath>,
) -> HttpResponse {
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

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(&ctx.dao.db, path.project_id()),
        CollectionDao::db_select(&ctx.dao.db, path.collection_id()),
    ) {
        Ok(data) => data,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.id() != collection_data.project_id() {
        return Response::error(StatusCode::BAD_REQUEST, "Project ID does not match");
    }

    Response::data(
        StatusCode::OK,
        None,
        CollectionResJson::new(
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
                        SchemaFieldModelJson::new(&value.kind().to_string(), value.required()),
                    )
                })
                .collect(),
            collection_data.indexes(),
        ),
    )
}

async fn update_one(
    ctx: web::Data<Context>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<UpdateOneCollectionReqPath>,
    data: web::Json<UpdateOneCollectionReqJson>,
) -> HttpResponse {
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

    let (project_data, mut collection_data) = match tokio::try_join!(
        ProjectDao::db_select(&ctx.dao.db, path.project_id()),
        CollectionDao::db_select(&ctx.dao.db, path.collection_id()),
    ) {
        Ok(data) => data,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.id() != collection_data.project_id() {
        return Response::error(StatusCode::BAD_REQUEST, "Project ID does not match");
    }

    if let Some(name) = data.name() {
        collection_data.set_name(name);
    }

    if let Some(schema_field) = data.schema_fields() {
        let mut schema_fields = HashMap::with_capacity(schema_field.len());
        for (key, value) in schema_field.iter() {
            schema_fields.insert(
                key.to_owned(),
                SchemaFieldModel::new(
                    match &SchemaFieldKind::from_str(value.kind()) {
                        Ok(kind) => kind,
                        Err(err) => {
                            return Response::error(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                &err.to_string(),
                            )
                        }
                    },
                    value.required(),
                ),
            );
        }
        collection_data.set_schema_fields(&schema_fields);
    }

    if let Some(indexes) = data.indexes() {
        collection_data.set_indexes(indexes);
    }

    if !data.is_all_none() {
        if let Err(err) = collection_data.db_update(&ctx.dao.db).await {
            return Response::error(StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
        }
    }

    Response::data(
        StatusCode::OK,
        None,
        CollectionResJson::new(
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
                        SchemaFieldModelJson::new(&value.kind().to_string(), value.required()),
                    )
                })
                .collect(),
            collection_data.indexes(),
        ),
    )
}

async fn delete_one(
    ctx: web::Data<Context>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<DeleteOneCollectionReqPath>,
) -> HttpResponse {
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

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(&ctx.dao.db, path.project_id()),
        CollectionDao::db_select(&ctx.dao.db, path.collection_id()),
    ) {
        Ok(data) => data,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.id() != collection_data.project_id() {
        return Response::error(StatusCode::BAD_REQUEST, "Project ID does not match");
    }

    if let Err(err) = CollectionDao::db_delete(&ctx.dao.db, path.collection_id()).await {
        return Response::error(StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        StatusCode::OK,
        None,
        DeleteCollectionResJson::new(collection_data.id()),
    )
}

async fn find_many(
    ctx: web::Data<Context>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<FindManyCollectionReqPath>,
) -> HttpResponse {
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

    let project_data = match ProjectDao::db_select(&ctx.dao.db, path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error(StatusCode::FORBIDDEN, "This project does not belong to you");
    }

    let collections_data =
        match CollectionDao::db_select_by_project_id(&ctx.dao.db, path.project_id()).await {
            Ok(data) => data,
            Err(err) => return Response::error(StatusCode::BAD_REQUEST, &err.to_string()),
        };

    Response::data(
        StatusCode::OK,
        Some(PaginationRes::new(
            &(collections_data.len() as i64),
            &(collections_data.len() as i64),
            &1,
            &(collections_data.len() as i64),
        )),
        collections_data
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
                                SchemaFieldModelJson::new(
                                    &value.kind().to_string(),
                                    value.required(),
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
