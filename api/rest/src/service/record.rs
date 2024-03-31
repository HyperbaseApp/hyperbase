use actix_web::{http::StatusCode, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use hb_api_websocket::server::{Message as WebSocketMessage, MessageKind as WebSocketMessageKind};
use hb_dao::{
    admin::AdminDao,
    collection::CollectionDao,
    collection_rule::CollectionPermission,
    project::ProjectDao,
    record::{RecordDao, RecordFilters, RecordOrder, RecordPagination},
    token::TokenDao,
    value::{ColumnKind, ColumnValue},
};
use hb_token_jwt::kind::JwtTokenKind;
use uuid::Uuid;

use crate::{
    context::ApiRestCtx,
    model::{
        record::{
            DeleteOneRecordReqPath, DeleteRecordResJson, FindManyRecordReqJson,
            FindManyRecordReqPath, FindOneRecordReqPath, FindOneRecordReqQuery,
            InsertOneRecordReqJson, InsertOneRecordReqPath, RecordResJson, UpdateOneRecordReqJson,
            UpdateOneRecordReqPath,
        },
        PaginationRes, Response,
    },
};

pub fn record_api(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/project/{project_id}/collection/{collection_id}/record",
        web::post().to(insert_one),
    )
    .route(
        "/project/{project_id}/collection/{collection_id}/record/{record_id}",
        web::get().to(find_one),
    )
    .route(
        "/project/{project_id}/collection/{collection_id}/record/{record_id}",
        web::patch().to(update_one),
    )
    .route(
        "/project/{project_id}/collection/{collection_id}/record/{record_id}",
        web::delete().to(delete_one),
    )
    .route(
        "/project/{project_id}/collection/{collection_id}/records",
        web::post().to(find_many),
    );
}

async fn insert_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<InsertOneRecordReqPath>,
    data: web::Json<InsertOneRecordReqJson>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::Admin => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::UserAnonymous | JwtTokenKind::User => {
            match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
                Ok(data) => (*data.admin_id(), Some(data)),
                Err(err) => {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        &format!("Failed to get token data: {err}"),
                    )
                }
            }
        }
    };

    if let Some(token_data) = &token_data {
        if !token_data
            .is_allow_insert_record(ctx.dao().db(), path.collection_id())
            .await
        {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to write data to this collection",
            );
        }
    }

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        CollectionDao::db_select(ctx.dao().db(), path.collection_id())
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

    for field_name in data.keys() {
        if field_name == "_created_by" {
            if *token_claim.kind() == JwtTokenKind::Admin {
                continue;
            } else {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    "Must be logged in using password-based login to insert '_created_by' field",
                );
            }
        }
        if !collection_data.schema_fields().contains_key(field_name) {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                &format!("Field '{field_name}' is not exist in the collection"),
            );
        }
    }

    let created_by = if let Some(created_by) = data.get("_created_by") {
        match created_by.as_str() {
            Some(created_by) => match Uuid::parse_str(created_by) {
                Ok(created_by) => created_by,
                Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
            },
            None => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Invalid '_created_by' field"),
                )
            }
        }
    } else if *token_claim.kind() == JwtTokenKind::Admin {
        admin_id
    } else if let Some(user_claim) = token_claim.user() {
        let collection_data =
            match CollectionDao::db_select(ctx.dao().db(), user_claim.collection_id()).await {
                Ok(data) => data,
                Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
            };
        let user_data = match RecordDao::db_select(
            ctx.dao().db(),
            user_claim.id(),
            &None,
            &HashSet::from_iter(["_id"]),
            &collection_data,
        )
        .await
        {
            Ok(data) => data,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

        let mut user_id = None;
        if let Some(id) = user_data.get("_id") {
            if let ColumnValue::Uuid(id) = id {
                if let Some(id) = id {
                    user_id = Some(*id);
                }
            }
        }

        if let Some(user_id) = user_id {
            user_id
        } else {
            return Response::error_raw(&StatusCode::BAD_REQUEST, "User doesn't found");
        }
    } else {
        *token_claim.id()
    };

    let mut record_data = RecordDao::new(&created_by, collection_data.id(), &Some(data.len()));
    for (field_name, field_props) in collection_data.schema_fields() {
        if let Some(value) = data.get(field_name) {
            if !value.is_null() {
                record_data.upsert(
                    field_name,
                    &match ColumnValue::from_serde_json(field_props.kind(), value) {
                        Ok(value) => value,
                        Err(err) => {
                            return Response::error_raw(
                                &StatusCode::BAD_REQUEST,
                                &format!("Error in field '{field_name}': {err}"),
                            )
                        }
                    },
                );
                continue;
            }
        }
        if *field_props.required() {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                &format!("Value for '{field_name}' is required"),
            );
        } else {
            record_data.upsert(field_name, &ColumnValue::none(field_props.kind()));
        }
    }

    if let Err(err) = record_data.db_insert(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    let mut record = HashMap::with_capacity(record_data.len());
    for (key, value) in record_data.data() {
        let value = match value.to_serde_json() {
            Ok(value) => value,
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("Error in field '{key}': {err}"),
                )
            }
        };
        record.insert(key.to_owned(), value);
    }

    let record_pub = record.clone();
    tokio::spawn((|| async move {
        let record_id = Uuid::parse_str(record_pub["_id"].as_str().unwrap()).unwrap();
        let created_by = Uuid::parse_str(record_pub["_created_by"].as_str().unwrap()).unwrap();

        let record = match serde_json::to_value(&record_pub) {
            Ok(value) => value,
            Err(err) => {
                hb_log::error(
                    None,
                    &format!("ApiRestServer: Error when serializing record {record_id}: {err}"),
                );
                return;
            }
        };

        if let Err(err) = ctx.websocket().handler().clone().broadcast(
            *collection_data.id(),
            WebSocketMessage::new(
                *collection_data.id(),
                record_id,
                created_by,
                WebSocketMessageKind::InsertOne,
                record,
            ),
        ) {
            hb_log::error(
                None,
                &format!(
                    "ApiRestServer: Error when broadcasting insert_one record {} to websocket: {}",
                    record_pub["_id"], err
                ),
            );
            return;
        }
    })());

    Response::data(&StatusCode::CREATED, &None, &RecordResJson::new(&record))
}

async fn find_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<FindOneRecordReqPath>,
    query: web::Query<FindOneRecordReqQuery>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::Admin => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::UserAnonymous | JwtTokenKind::User => {
            match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
                Ok(data) => (*data.admin_id(), Some(data)),
                Err(err) => {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        &format!("Failed to get token data: {err}"),
                    )
                }
            }
        }
    };

    let rule_find_one = if let Some(token_data) = &token_data {
        if let Some(rule) = token_data
            .is_allow_find_one_record(ctx.dao().db(), path.collection_id())
            .await
        {
            Some(rule)
        } else {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to read this record",
            );
        }
    } else {
        None
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

    let created_by = if *token_claim.kind() == JwtTokenKind::Admin {
        None
    } else if let Some(rule) = rule_find_one {
        match rule {
            CollectionPermission::All => None,
            CollectionPermission::SelfMade => match token_claim.user() {
                Some(user_claim) => {
                    let collection_data =
                        match CollectionDao::db_select(ctx.dao().db(), user_claim.collection_id())
                            .await
                        {
                            Ok(data) => data,
                            Err(err) => {
                                return Response::error_raw(
                                    &StatusCode::BAD_REQUEST,
                                    &err.to_string(),
                                )
                            }
                        };
                    let user_data = match RecordDao::db_select(
                        ctx.dao().db(),
                        user_claim.id(),
                        &None,
                        &HashSet::from_iter(["_id"]),
                        &collection_data,
                    )
                    .await
                    {
                        Ok(data) => data,
                        Err(err) => {
                            return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string())
                        }
                    };

                    let mut user_id = None;
                    if let Some(id) = user_data.get("_id") {
                        if let ColumnValue::Uuid(id) = id {
                            if let Some(id) = id {
                                user_id = Some(*id);
                            }
                        }
                    }

                    if user_id.is_none() {
                        return Response::error_raw(&StatusCode::BAD_REQUEST, "User not found");
                    }

                    user_id
                }
                None => Some(*token_claim.id()),
            },
            CollectionPermission::None => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    "User doesn't have permission to read this record",
                )
            }
        }
    } else {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            "User doesn't have permission to read this record",
        );
    };

    let fields = match query.fields() {
        Some(origin_fields) => {
            let mut fields = HashSet::with_capacity(origin_fields.len());
            for field in origin_fields {
                if collection_data.schema_fields().contains_key(field) || field == "_id" {
                    fields.insert(field.as_str());
                } else {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        &format!("Field '{field}' is not exist in the collection"),
                    );
                }
            }
            fields
        }
        None => HashSet::new(),
    };

    let record_data = match RecordDao::db_select(
        ctx.dao().db(),
        path.record_id(),
        &created_by,
        &fields,
        &collection_data,
    )
    .await
    {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let mut record = HashMap::with_capacity(record_data.len());
    for (key, value) in record_data.data() {
        let value = match value.to_serde_json() {
            Ok(value) => value,
            Err(err) => {
                return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
            }
        };
        record.insert(key.to_owned(), value);
    }

    Response::data(&StatusCode::OK, &None, &RecordResJson::new(&record))
}

async fn update_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<UpdateOneRecordReqPath>,
    data: web::Json<UpdateOneRecordReqJson>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::Admin => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::UserAnonymous | JwtTokenKind::User => {
            match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
                Ok(data) => (*data.admin_id(), Some(data)),
                Err(err) => {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        &format!("Failed to get token data: {err}"),
                    )
                }
            }
        }
    };

    let rule_update_one = if let Some(token_data) = &token_data {
        if let Some(rule) = token_data
            .is_allow_update_record(ctx.dao().db(), path.collection_id())
            .await
        {
            Some(rule)
        } else {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to update this record",
            );
        }
    } else {
        None
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

    let created_by = if *token_claim.kind() == JwtTokenKind::Admin {
        None
    } else if let Some(rule) = rule_update_one {
        match rule {
            CollectionPermission::All => None,
            CollectionPermission::SelfMade => match token_claim.user() {
                Some(user_claim) => {
                    let collection_data =
                        match CollectionDao::db_select(ctx.dao().db(), user_claim.collection_id())
                            .await
                        {
                            Ok(data) => data,
                            Err(err) => {
                                return Response::error_raw(
                                    &StatusCode::BAD_REQUEST,
                                    &err.to_string(),
                                )
                            }
                        };
                    let user_data = match RecordDao::db_select(
                        ctx.dao().db(),
                        user_claim.id(),
                        &None,
                        &HashSet::from_iter(["_id"]),
                        &collection_data,
                    )
                    .await
                    {
                        Ok(data) => data,
                        Err(err) => {
                            return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string())
                        }
                    };

                    let mut user_id = None;
                    if let Some(id) = user_data.get("_id") {
                        if let ColumnValue::Uuid(id) = id {
                            if let Some(id) = id {
                                user_id = Some(*id);
                            }
                        }
                    }

                    if user_id.is_none() {
                        return Response::error_raw(&StatusCode::BAD_REQUEST, "User not found");
                    }

                    user_id
                }
                None => Some(*token_claim.id()),
            },
            CollectionPermission::None => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    "User doesn't have permission to read this record",
                )
            }
        }
    } else {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            "User doesn't have permission to update this record",
        );
    };

    for field_name in data.keys() {
        if field_name == "_created_by" {
            if *token_claim.kind() == JwtTokenKind::Admin {
                continue;
            } else {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    "Must be logged in using password-based login to update '_created_by' field",
                );
            }
        }
        if !collection_data.schema_fields().contains_key(field_name) {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                &format!("Field '{field_name}' is not exist in the collection"),
            );
        }
    }

    let mut record_data = match RecordDao::db_select(
        ctx.dao().db(),
        path.record_id(),
        &created_by,
        &HashSet::new(),
        &collection_data,
    )
    .await
    {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if let Some(created_by) = data.get("_created_by") {
        if !created_by.is_null() {
            record_data.upsert(
                "_created_by",
                &match ColumnValue::from_serde_json(&ColumnKind::Uuid, created_by) {
                    Ok(value) => value,
                    Err(err) => {
                        return Response::error_raw(
                            &StatusCode::BAD_REQUEST,
                            &format!("Error in field '_created_by': {err}"),
                        )
                    }
                },
            )
        }
    }

    for (field_name, field_props) in collection_data.schema_fields() {
        if let Some(value) = data.get(field_name) {
            if value.is_null() {
                if *field_props.required() {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        &format!("Value for '{field_name}' is required"),
                    );
                }
            }
            record_data.upsert(
                field_name,
                &match ColumnValue::from_serde_json(field_props.kind(), value) {
                    Ok(value) => value,
                    Err(err) => {
                        return Response::error_raw(
                            &StatusCode::BAD_REQUEST,
                            &format!("Error in field '{field_name}': {err}"),
                        )
                    }
                },
            );
        }
    }

    if let Err(err) = record_data.db_update(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    let mut record = HashMap::with_capacity(record_data.len());
    for (key, value) in record_data.data() {
        let value = match value.to_serde_json() {
            Ok(value) => value,
            Err(err) => {
                return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
            }
        };
        record.insert(key.to_owned(), value);
    }

    let record_id = path.record_id().clone();
    let record_pub = record.clone();
    tokio::spawn((|| async move {
        let created_by = Uuid::parse_str(record_pub["_created_by"].as_str().unwrap()).unwrap();

        let record = match serde_json::to_value(&record_pub) {
            Ok(value) => value,
            Err(err) => {
                hb_log::error(
                    None,
                    &format!("ApiRestServer: Error when serializing record {record_id}: {err}"),
                );
                return;
            }
        };

        if let Err(err) = ctx.websocket().handler().clone().broadcast(
            *collection_data.id(),
            WebSocketMessage::new(
                *collection_data.id(),
                record_id,
                created_by,
                WebSocketMessageKind::UpdateOne,
                record,
            ),
        ) {
            hb_log::error(
                None,
                &format!(
                    "ApiRestServer: Error when broadcasting update_one record {} to websocket: {}",
                    record_pub["_id"], err
                ),
            );
            return;
        }
    })());

    Response::data(&StatusCode::OK, &None, &RecordResJson::new(&record))
}

async fn delete_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<DeleteOneRecordReqPath>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::Admin => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::UserAnonymous | JwtTokenKind::User => {
            match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
                Ok(data) => (*data.admin_id(), Some(data)),
                Err(err) => {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        &format!("Failed to get token data: {err}"),
                    )
                }
            }
        }
    };

    let rule_delete_one = if let Some(token_data) = &token_data {
        if let Some(rule) = token_data
            .is_allow_delete_record(ctx.dao().db(), path.collection_id())
            .await
        {
            Some(rule)
        } else {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to delete this record",
            );
        }
    } else {
        None
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

    let created_by = if *token_claim.kind() == JwtTokenKind::Admin {
        None
    } else if let Some(rule) = rule_delete_one {
        match rule {
            CollectionPermission::All => None,
            CollectionPermission::SelfMade => match token_claim.user() {
                Some(user_claim) => {
                    let collection_data =
                        match CollectionDao::db_select(ctx.dao().db(), user_claim.collection_id())
                            .await
                        {
                            Ok(data) => data,
                            Err(err) => {
                                return Response::error_raw(
                                    &StatusCode::BAD_REQUEST,
                                    &err.to_string(),
                                )
                            }
                        };
                    let user_data = match RecordDao::db_select(
                        ctx.dao().db(),
                        user_claim.id(),
                        &None,
                        &HashSet::from_iter(["_id"]),
                        &collection_data,
                    )
                    .await
                    {
                        Ok(data) => data,
                        Err(err) => {
                            return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string())
                        }
                    };

                    let mut user_id = None;
                    if let Some(id) = user_data.get("_id") {
                        if let ColumnValue::Uuid(id) = id {
                            if let Some(id) = id {
                                user_id = Some(*id);
                            }
                        }
                    }

                    if user_id.is_none() {
                        return Response::error_raw(&StatusCode::BAD_REQUEST, "User not found");
                    }

                    user_id
                }
                None => Some(*token_claim.id()),
            },
            CollectionPermission::None => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    "User doesn't have permission to read this record",
                )
            }
        }
    } else {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            "User doesn't have permission to delete this record",
        );
    };

    let mut fields = HashSet::with_capacity(1);
    fields.insert("_created_by");

    let record_data = match RecordDao::db_select(
        ctx.dao().db(),
        path.record_id(),
        &created_by,
        &fields,
        &collection_data,
    )
    .await
    {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if let Err(err) = RecordDao::db_delete(
        ctx.dao().db(),
        collection_data.id(),
        path.record_id(),
        &created_by,
    )
    .await
    {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    let record_id = path.record_id().clone();
    tokio::spawn((|| async move {
        let created_by = Uuid::parse_str(
            record_data.data()["_created_by"]
                .to_serde_json()
                .unwrap()
                .as_str()
                .unwrap(),
        )
        .unwrap();

        let record_id_val = match serde_json::to_value(&record_id) {
            Ok(value) => value,
            Err(err) => {
                hb_log::error(
                    None,
                    &format!("ApiRestServer: Error when serializing record {record_id}: {err}"),
                );
                return;
            }
        };

        if let Err(err) = ctx.websocket().handler().clone().broadcast(
            *collection_data.id(),
            WebSocketMessage::new(
                *collection_data.id(),
                record_id,
                created_by,
                WebSocketMessageKind::DeleteOne,
                record_id_val,
            ),
        ) {
            hb_log::error(
                None,
                &format!(
                    "ApiRestServer: Error when broadcasting delete_one record {record_id} to websocket: {err}"
                ),
            );
            return;
        }
    })());

    Response::data(
        &StatusCode::OK,
        &None,
        &DeleteRecordResJson::new(path.record_id()),
    )
}

async fn find_many(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<FindManyRecordReqPath>,
    query_data: web::Json<FindManyRecordReqJson>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::Admin => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::UserAnonymous | JwtTokenKind::User => {
            match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
                Ok(data) => (*data.admin_id(), Some(data)),
                Err(err) => {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        &format!("Failed to get token data: {err}"),
                    )
                }
            }
        }
    };

    let rule_find_many = if let Some(token_data) = &token_data {
        if let Some(rule) = token_data
            .is_allow_find_many_records(ctx.dao().db(), path.collection_id())
            .await
        {
            Some(rule)
        } else {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to read these records",
            );
        }
    } else {
        None
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

    let created_by = if *token_claim.kind() == JwtTokenKind::Admin {
        None
    } else if let Some(rule) = rule_find_many {
        match rule {
            CollectionPermission::All => None,
            CollectionPermission::SelfMade => match token_claim.user() {
                Some(user_claim) => {
                    let collection_data =
                        match CollectionDao::db_select(ctx.dao().db(), user_claim.collection_id())
                            .await
                        {
                            Ok(data) => data,
                            Err(err) => {
                                return Response::error_raw(
                                    &StatusCode::BAD_REQUEST,
                                    &err.to_string(),
                                )
                            }
                        };
                    let user_data = match RecordDao::db_select(
                        ctx.dao().db(),
                        user_claim.id(),
                        &None,
                        &HashSet::from_iter(["_id"]),
                        &collection_data,
                    )
                    .await
                    {
                        Ok(data) => data,
                        Err(err) => {
                            return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string())
                        }
                    };

                    let mut user_id = None;
                    if let Some(id) = user_data.get("_id") {
                        if let ColumnValue::Uuid(id) = id {
                            if let Some(id) = id {
                                user_id = Some(*id);
                            }
                        }
                    }

                    if user_id.is_none() {
                        return Response::error_raw(&StatusCode::BAD_REQUEST, "User not found");
                    }

                    user_id
                }
                None => Some(*token_claim.id()),
            },
            CollectionPermission::None => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    "User doesn't have permission to read this record",
                )
            }
        }
    } else {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "User doesn't have permission to read these records",
        );
    };

    let fields = match query_data.fields() {
        Some(origin_fields) => {
            let mut fields = HashSet::with_capacity(origin_fields.len());
            for field in origin_fields {
                if collection_data.schema_fields().contains_key(field) || field == "_id" {
                    fields.insert(field.as_str());
                } else {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        &format!("Field '{field}' is not exist in the collection"),
                    );
                }
            }
            fields
        }
        None => HashSet::new(),
    };
    let filters = match query_data.filters() {
        Some(filter) => match filter.to_dao(&collection_data) {
            Ok(filter) => filter,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        },
        None => RecordFilters::new(&Vec::new()),
    };
    let groups = match query_data.groups() {
        Some(origin_groups) => {
            let mut groups = Vec::with_capacity(origin_groups.len());
            for field in origin_groups {
                if collection_data.schema_fields().contains_key(field) || field == "_id" {
                    groups.push(field.as_str());
                } else {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        &format!("Field '{field}' is not exist in the collection"),
                    );
                }
            }
            groups
        }
        None => Vec::new(),
    };
    let orders = match query_data.orders() {
        Some(order) => {
            let mut orders = Vec::with_capacity(order.len());
            for o in order {
                if collection_data.schema_fields().contains_key(o.field()) || o.field() == "_id" {
                    orders.push(RecordOrder::new(o.field(), o.kind()));
                } else {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        &format!("Field '{}' is not exist in the collection", o.field()),
                    );
                }
            }
            orders
        }
        None => Vec::new(),
    };
    let pagination = RecordPagination::new(query_data.limit());
    let (records_data, total) = match RecordDao::db_select_many(
        ctx.dao().db(),
        &fields,
        &collection_data,
        &created_by,
        &filters,
        &groups,
        &orders,
        &pagination,
    )
    .await
    {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let mut records = Vec::with_capacity(records_data.len());
    for record_data in &records_data {
        let mut record = HashMap::with_capacity(record_data.len());
        for (key, value) in record_data.data() {
            let value = match value.to_serde_json() {
                Ok(value) => value,
                Err(err) => {
                    return Response::error_raw(
                        &StatusCode::INTERNAL_SERVER_ERROR,
                        &err.to_string(),
                    )
                }
            };
            record.insert(key.to_owned(), value);
        }
        records.push(record);
    }

    let total = match usize::try_from(total) {
        Ok(data) => data,
        Err(err) => {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
        }
    };

    Response::data(
        &StatusCode::OK,
        &Some(PaginationRes::new(&records_data.len(), &total)),
        &records,
    )
}
