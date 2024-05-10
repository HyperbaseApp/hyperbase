use actix_web::{http::StatusCode, web, HttpRequest, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use ahash::{HashMap, HashMapExt};
use chrono::Utc;
use hb_api_websocket::{message::Target, session::UserSession};
use hb_dao::{
    admin::AdminDao,
    change::{ChangeDao, ChangeState, ChangeTable},
    collection::{CollectionDao, SchemaFieldProps},
    project::ProjectDao,
    token::TokenDao,
    value::ColumnKind,
};
use hb_token_jwt::claim::ClaimId;

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

    let mut schema_fields = HashMap::with_capacity(data.schema_fields().len());
    for (field, props) in data.schema_fields().iter() {
        if field.is_empty() {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                &format!("Field name in schema_fields can't be empty string"),
            );
        }
        if field.starts_with("_") || !field.chars().all(|c| c == '_' || (c >= 'a' && c <= 'z')) {
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
                &props.unique().unwrap_or(false),
                &props.indexed().unwrap_or(false),
                &props.auth_column().unwrap_or(false),
                &props.hidden().unwrap_or(false),
            ),
        );
    }

    let collection_data = CollectionDao::new(
        path.project_id(),
        data.name(),
        &schema_fields,
        data.opt_auth_column_id(),
        data.opt_ttl(),
    );
    if let Err(err) = collection_data.db_insert(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    let change_data = ChangeDao::new(
        &ChangeTable::Collection,
        collection_data.id(),
        &ChangeState::Insert,
        &collection_data.created_at(),
    );
    if let Err(err) = change_data.db_upsert(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    if let Some(internal_broadcast) = ctx.internal_broadcast() {
        let internal_broadcast = internal_broadcast.clone();
        tokio::spawn((|| async move {
            if let Err(err) = internal_broadcast.broadcast(&change_data).await {
                hb_log::error(
                    None,
                    &format!(
                        "[ApiRestServer] Error when broadcasting insert_one collection to remote peer: {err}"
                    ),
                );
            }
        })());
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
                            &Some(*props.unique()),
                            &Some(*props.indexed()),
                            &Some(*props.auth_column()),
                            &Some(*props.hidden()),
                        ),
                    )
                })
                .collect(),
            collection_data.opt_auth_column_id(),
            collection_data.opt_ttl(),
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

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        CollectionDao::db_select(ctx.dao().db(), path.collection_id()),
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
                            &Some(*value.unique()),
                            &Some(*value.indexed()),
                            &Some(*value.auth_column()),
                            &Some(*value.hidden()),
                        ),
                    )
                })
                .collect(),
            collection_data.opt_auth_column_id(),
            collection_data.opt_ttl(),
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

    let (admin_id, token) = match token_claim.id() {
        ClaimId::Admin(id) => match AdminDao::db_select(ctx.dao().db(), id).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::UNAUTHORIZED,
                    &format!("Failed to get admin data: {err}"),
                )
            }
        },
        ClaimId::Token(token_id, user) => {
            let token_data = match TokenDao::db_select(ctx.dao().db(), token_id).await {
                Ok(data) => data,
                Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
            };
            (
                *token_data.admin_id(),
                Some((
                    *token_id,
                    match user {
                        Some(user) => Some(*user.id()),
                        None => None,
                    },
                )),
            )
        }
    };

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        CollectionDao::db_select(ctx.dao().db(), path.collection_id()),
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
                match token {
                    Some((token_id, user_id)) => UserSession::Token(token_id, user_id),
                    None => UserSession::Admin(admin_id),
                },
                Target::Collection(*collection_data.id()),
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

    let (project_data, mut collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        CollectionDao::db_select(ctx.dao().db(), path.collection_id()),
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
                    &props.unique().unwrap_or(false),
                    &props.indexed().unwrap_or(false),
                    &props.auth_column().unwrap_or(false),
                    &props.hidden().unwrap_or(false),
                ),
            );
        }
        collection_data.update_schema_fields(&schema_fields);
    }

    if let Some(opt_auth_column_id) = data.opt_auth_column_id() {
        collection_data.set_opt_auth_column_id(opt_auth_column_id);
    }

    if let Some(opt_ttl) = data.opt_ttl() {
        collection_data.set_opt_ttl(opt_ttl);
    }

    if !data.is_all_none() {
        if let Err(err) = collection_data.db_update(ctx.dao().db()).await {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
        }

        let change_data = ChangeDao::new(
            &ChangeTable::Collection,
            collection_data.id(),
            &ChangeState::Update,
            &collection_data.updated_at(),
        );
        if let Err(err) = change_data.db_upsert(ctx.dao().db()).await {
            return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
        }

        if let Some(internal_broadcast) = ctx.internal_broadcast() {
            let internal_broadcast = internal_broadcast.clone();
            tokio::spawn((|| async move {
                if let Err(err) = internal_broadcast.broadcast(&change_data).await {
                    hb_log::error(
                        None,
                        &format!(
                            "[ApiRestServer] Error when broadcasting update_one collection to remote peer: {err}"
                        ),
                    );
                }
            })());
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
                            &Some(*value.unique()),
                            &Some(*value.indexed()),
                            &Some(*value.auth_column()),
                            &Some(*value.hidden()),
                        ),
                    )
                })
                .collect(),
            collection_data.opt_auth_column_id(),
            collection_data.opt_ttl(),
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

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        CollectionDao::db_select(ctx.dao().db(), path.collection_id()),
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

    if project_data.id() != collection_data.project_id() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Project id does not match");
    }

    let deleted_at = Utc::now();

    if let Err(err) = CollectionDao::db_delete(ctx.dao().db(), path.collection_id()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    let change_data = ChangeDao::new(
        &ChangeTable::Collection,
        collection_data.id(),
        &ChangeState::Delete,
        &deleted_at,
    );
    if let Err(err) = change_data.db_upsert(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    if let Some(internal_broadcast) = ctx.internal_broadcast() {
        let internal_broadcast = internal_broadcast.clone();
        tokio::spawn((|| async move {
            if let Err(err) = internal_broadcast.broadcast(&change_data).await {
                hb_log::error(
                    None,
                    &format!(
                        "[ApiRestServer] Error when broadcasting delete_one collection to remote peer: {err}"
                    ),
                );
            }
        })());
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
                                    &Some(*value.unique()),
                                    &Some(*value.indexed()),
                                    &Some(*value.auth_column()),
                                    &Some(*value.hidden()),
                                ),
                            )
                        })
                        .collect(),
                    data.opt_auth_column_id(),
                    data.opt_ttl(),
                )
            })
            .collect::<Vec<_>>(),
    )
}
