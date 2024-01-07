use actix_web::{http::StatusCode, web, HttpResponse};
use ahash::{HashMap, HashMapExt};
use hb_dao::{
    admin::AdminDao,
    collection::CollectionDao,
    project::ProjectDao,
    record::{RecordDao, RecordFilters, RecordOrder, RecordPagination},
    token::TokenDao,
    value::{ColumnKind, ColumnValue},
};
use hb_token_jwt::kind::JwtTokenKind;

use crate::{
    context::ApiRestCtx,
    model::{
        record::{
            DeleteOneRecordReqPath, DeleteRecordResJson, FindManyRecordReqJson,
            FindManyRecordReqPath, FindOneRecordReqPath, InsertOneRecordReqJson,
            InsertOneRecordReqPath, RecordResJson, UpdateOneRecordReqJson, UpdateOneRecordReqPath,
        },
        PaginationRes, Response, TokenReqHeader,
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
    token: web::Header<TokenReqHeader>,
    path: web::Path<InsertOneRecordReqPath>,
    data: web::Json<InsertOneRecordReqJson>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error_raw(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.admin_id(), Some(data)),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get token data: {err}"),
                )
            }
        },
    };

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
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Project ID does not match");
    }

    if let Some(token_data) = &token_data {
        if !token_data.is_allow_insert(path.collection_id()) {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to write data to this collection",
            );
        }
    }

    for field_name in data.keys() {
        if !collection_data.schema_fields().contains_key(field_name) {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                &format!("Field '{field_name}' is not exist in the collection"),
            );
        }
    }

    let mut record_data = RecordDao::new(collection_data.id(), &Some(data.len()));
    for (field_name, field_props) in collection_data.schema_fields() {
        if let Some(value) = data.get(field_name) {
            if !value.is_null() {
                if let Some(value) = value.as_str() {
                    if value == "$request.auth.id" {
                        if *field_props.kind() != ColumnKind::Uuid {
                            return Response::error_raw(
                                &StatusCode::BAD_REQUEST,
                                "Field for storing '$request.auth.id' must be of type 'uuid'",
                            );
                        }
                        record_data.upsert(field_name, &ColumnValue::Uuid(Some(*token_claim.id())));
                        continue;
                    }
                }
                record_data.upsert(
                    field_name,
                    &match ColumnValue::from_serde_json(field_props.kind(), value) {
                        Ok(value) => value,
                        Err(err) => {
                            return Response::error_raw(
                                &StatusCode::BAD_REQUEST,
                                &format!("Error in field '{}': {}", field_name, err),
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
                return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
            }
        };
        record.insert(key.to_owned(), value);
    }

    Response::data(&StatusCode::CREATED, &None, &RecordResJson::new(&record))
}

async fn find_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<FindOneRecordReqPath>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error_raw(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.admin_id(), Some(data)),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get token data: {err}"),
                )
            }
        },
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
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Project ID does not match");
    }

    if let Some(token_data) = &token_data {
        if !token_data.is_allow_find_one(path.collection_id()) {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to read this record",
            );
        }
    }

    let record_data =
        match RecordDao::db_select(ctx.dao().db(), &collection_data, path.record_id()).await {
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
    token: web::Header<TokenReqHeader>,
    path: web::Path<UpdateOneRecordReqPath>,
    data: web::Json<UpdateOneRecordReqJson>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error_raw(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.admin_id(), Some(data)),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get token data: {err}"),
                )
            }
        },
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
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Project ID does not match");
    }

    if let Some(token_data) = &token_data {
        if !token_data.is_allow_update(path.collection_id()) {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to update this record",
            );
        }
    }

    for field_name in data.keys() {
        if !collection_data.schema_fields().contains_key(field_name) {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                &format!("Field '{field_name}' is not exist in the collection"),
            );
        }
    }

    let mut record_data =
        match RecordDao::db_select(ctx.dao().db(), &collection_data, path.record_id()).await {
            Ok(data) => data,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };
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
            if let Some(value) = value.as_str() {
                if value == "$request.auth.id" {
                    if *field_props.kind() != ColumnKind::Uuid {
                        return Response::error_raw(
                            &StatusCode::BAD_REQUEST,
                            "Field for storing '$request.auth.id' must be of type 'uuid'",
                        );
                    }
                    record_data.upsert(field_name, &ColumnValue::Uuid(Some(*token_claim.id())));
                    continue;
                }
            }
            record_data.upsert(
                field_name,
                &match ColumnValue::from_serde_json(field_props.kind(), value) {
                    Ok(value) => value,
                    Err(err) => {
                        return Response::error_raw(
                            &StatusCode::BAD_REQUEST,
                            &format!("Error in field '{}': {}", field_name, err),
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

    Response::data(&StatusCode::OK, &None, &RecordResJson::new(&record))
}

async fn delete_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<DeleteOneRecordReqPath>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error_raw(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.admin_id(), Some(data)),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get token data: {err}"),
                )
            }
        },
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
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Project ID does not match");
    }

    if let Some(token_data) = &token_data {
        if !token_data.is_allow_delete(path.collection_id()) {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to delete this record",
            );
        }
    }

    if let Err(err) =
        RecordDao::db_delete(ctx.dao().db(), collection_data.id(), path.record_id()).await
    {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &DeleteRecordResJson::new(path.record_id()),
    )
}

async fn find_many(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<FindManyRecordReqPath>,
    query_data: web::Json<FindManyRecordReqJson>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error_raw(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.admin_id(), Some(data)),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get token data: {err}"),
                )
            }
        },
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
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Project ID does not match");
    }

    if let Some(token_data) = &token_data {
        if !token_data.is_allow_find_many(path.collection_id()) {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to read these records",
            );
        }
    }

    let filters = match query_data.filter() {
        Some(filter) => match filter.to_dao(&collection_data) {
            Ok(filter) => filter,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        },
        None => RecordFilters::new(&Vec::new()),
    };
    let groups = match query_data.group() {
        Some(group) => {
            let mut groups = Vec::with_capacity(group.len());
            for field in group {
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
    let orders = match query_data.order() {
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
        &collection_data,
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
