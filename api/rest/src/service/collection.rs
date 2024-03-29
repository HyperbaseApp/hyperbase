use actix_web::{http::StatusCode, web, HttpRequest, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use ahash::{HashMap, HashMapExt};
use hb_dao::{
    admin::AdminDao,
    collection::{CollectionDao, SchemaFieldProps},
    project::ProjectDao,
    token::TokenDao,
    value::ColumnKind,
};
use hb_token_jwt::kind::JwtTokenKind;

use crate::{
    context::ApiRestCtx,
    model::{
        collection::{
            CollectionResJson, DeleteCollectionResJson, DeleteOneCollectionReqPath,
            FindManyCollectionReqPath, FindOneCollectionReqPath, InsertOneCollectionReqJson,
            InsertOneCollectionReqPath, SchemaFieldPropsJson, SubscribeCollectionReqPath,
            SubscribeCollectionReqQuery, UpdateOneCollectionReqJson, UpdateOneCollectionReqPath,
        },
        PaginationRes, Response,
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
        "/project/{project_id}/collection/{collection_id}/subscribe",
        web::get().to(subscribe),
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
    auth: BearerAuth,
    path: web::Path<InsertOneCollectionReqPath>,
    data: web::Json<InsertOneCollectionReqJson>,
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

    let mut schema_fields = HashMap::with_capacity(data.schema_fields().len());
    for (field, props) in data.schema_fields().iter() {
        if field.is_empty() {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                &format!("Field name in schema_fields can't be empty string"),
            );
        }
        if field.starts_with("_") || !field.chars().all(|c| c == '_' || ('a'..='z').contains(&c)) {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                &format!("Field '{field}' should only have lowercase English letters and an optional underscore (_) after the first character"),
            );
        }
        if props.indexed().is_some_and(|indexed| indexed)
            && !props.required().is_some_and(|required| required)
        {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                &format!("Field '{field}' must be required because it is in the indexes"),
            );
        }
        schema_fields.insert(
            field.to_owned(),
            SchemaFieldProps::new(
                &match ColumnKind::from_str(props.kind()) {
                    Ok(kind) => kind,
                    Err(err) => {
                        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string())
                    }
                },
                &props.required().unwrap_or(false),
                &props.indexed().unwrap_or(false),
                &props.auth_column().unwrap_or(false),
            ),
        );
    }

    let collection_data = CollectionDao::new(path.project_id(), data.name(), &schema_fields);
    if let Err(err) = collection_data.db_insert(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
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
                .map(|(field, props)| {
                    (
                        field.clone(),
                        SchemaFieldPropsJson::new(
                            props.kind().to_str(),
                            &Some(*props.required()),
                            &Some(*props.indexed()),
                            &Some(*props.auth_column()),
                        ),
                    )
                })
                .collect(),
        ),
    )
}

async fn find_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<FindOneCollectionReqPath>,
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

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        CollectionDao::db_select(ctx.dao().db(), path.collection_id()),
    ) {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if &admin_id != project_data.admin_id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if project_data.id() != collection_data.project_id() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Project id does not match");
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
                        SchemaFieldPropsJson::new(
                            value.kind().to_str(),
                            &Some(*value.required()),
                            &Some(*value.indexed()),
                            &Some(*value.auth_column()),
                        ),
                    )
                })
                .collect(),
        ),
    )
}

async fn subscribe(
    ctx: web::Data<ApiRestCtx>,
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<SubscribeCollectionReqPath>,
    query: web::Query<SubscribeCollectionReqQuery>,
) -> HttpResponse {
    let token = query.token();

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

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        CollectionDao::db_select(ctx.dao().db(), path.collection_id()),
    ) {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if &admin_id != project_data.admin_id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if project_data.id() != collection_data.project_id() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Project id does not match");
    }

    let (res, session, msg_stream) = match actix_ws_ng::handle(&req, stream) {
        Ok(res) => res,
        Err(err) => {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
        }
    };

    tokio::task::spawn_local((|| async move {
        let _ = ctx
            .websocket()
            .handler()
            .clone()
            .connection(
                if let Some(user) = token_claim.user() {
                    Some(*user.id())
                } else {
                    None
                },
                *token_claim.id(),
                *collection_data.id(),
                session,
                msg_stream,
            )
            .await;
    })());

    res
}

async fn update_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<UpdateOneCollectionReqPath>,
    data: web::Json<UpdateOneCollectionReqJson>,
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

    let (project_data, mut collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        CollectionDao::db_select(ctx.dao().db(), path.collection_id()),
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

    if project_data.id() != collection_data.project_id() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Project id does not match");
    }

    if let Some(name) = data.name() {
        collection_data.set_name(name);
    }

    if let Some(schema_field) = data.schema_fields() {
        let mut schema_fields = HashMap::with_capacity(schema_field.len());
        for (field, props) in schema_field.iter() {
            if field.is_empty() {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Field name in schema_fields can't be empty string"),
                );
            }
            if field.starts_with("_")
                || !field.chars().all(|c| c == '_' || ('a'..='z').contains(&c))
            {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Field '{field}' should only have lowercase English letters and an optional underscore (_) after the first character"),
                );
            }
            schema_fields.insert(
                field.to_owned(),
                SchemaFieldProps::new(
                    &match ColumnKind::from_str(props.kind()) {
                        Ok(kind) => kind,
                        Err(err) => {
                            return Response::error_raw(
                                &StatusCode::INTERNAL_SERVER_ERROR,
                                &err.to_string(),
                            )
                        }
                    },
                    &props.required().unwrap_or(false),
                    &props.indexed().unwrap_or(false),
                    &props.auth_column().unwrap_or(false),
                ),
            );
        }
        collection_data.update_schema_fields(&schema_fields);
    }

    if !data.is_all_none() {
        if let Err(err) = collection_data.db_update(ctx.dao().db()).await {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
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
                        SchemaFieldPropsJson::new(
                            value.kind().to_str(),
                            &Some(*value.required()),
                            &Some(*value.indexed()),
                            &Some(*value.auth_column()),
                        ),
                    )
                })
                .collect(),
        ),
    )
}

async fn delete_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<DeleteOneCollectionReqPath>,
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

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        CollectionDao::db_select(ctx.dao().db(), path.collection_id()),
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

    if project_data.id() != collection_data.project_id() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Project id does not match");
    }

    if let Err(err) = CollectionDao::db_delete(ctx.dao().db(), path.collection_id()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &DeleteCollectionResJson::new(collection_data.id()),
    )
}

async fn find_many(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<FindManyCollectionReqPath>,
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

    let collections_data = match CollectionDao::db_select_many_by_project_id(
        ctx.dao().db(),
        path.project_id(),
    )
    .await
    {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
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
                                SchemaFieldPropsJson::new(
                                    value.kind().to_str(),
                                    &Some(*value.required()),
                                    &Some(*value.indexed()),
                                    &Some(*value.auth_column()),
                                ),
                            )
                        })
                        .collect(),
                )
            })
            .collect::<Vec<_>>(),
    )
}
