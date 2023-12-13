use actix_web::{http::StatusCode, web, HttpResponse};
use ahash::{HashMap, HashMapExt};
use hb_dao::{
    admin::AdminDao,
    collection::CollectionDao,
    project::ProjectDao,
    record::{RecordColumnValue, RecordDao, RecordFilter, RecordPagination},
    token::TokenDao,
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
        None => return Response::error(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let admin_id = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => *data.id(),
            Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => *data.admin_id(),
            Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
        },
    };

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        CollectionDao::db_select(ctx.dao().db(), path.collection_id())
    ) {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if &admin_id != project_data.admin_id() {
        return Response::error(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if project_data.id() != collection_data.project_id() {
        return Response::error(&StatusCode::BAD_REQUEST, "Project ID does not match");
    }

    for field_name in data.keys() {
        if !collection_data.schema_fields().contains_key(field_name) {
            return Response::error(
                &StatusCode::BAD_REQUEST,
                &format!("Field {field_name} is not exist in the collection"),
            );
        }
    }

    let mut record_data = collection_data.new_record(&Some(data.len()));
    for (field_name, field_props) in collection_data.schema_fields() {
        if let Some(value) = data.get(field_name) {
            if !value.is_null() {
                record_data.upsert(
                    field_name,
                    &match RecordColumnValue::from_serde_json(field_props.kind(), value) {
                        Ok(value) => value,
                        Err(err) => {
                            return Response::error(
                                &StatusCode::BAD_REQUEST,
                                &format!("Error in field {}: {}", field_name, err),
                            )
                        }
                    },
                );
                continue;
            }
        }
        match field_props.required() {
            true => {
                return Response::error(
                    &StatusCode::BAD_REQUEST,
                    &format!("Value for {field_name} is required"),
                )
            }
            false => record_data.upsert(field_name, &RecordColumnValue::none(field_props.kind())),
        };
    }

    if let Err(err) = record_data.db_insert(ctx.dao().db()).await {
        return Response::error(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    let mut record = HashMap::with_capacity(record_data.len());
    for (key, value) in record_data.data() {
        let value = match value.to_serde_json() {
            Ok(value) => value,
            Err(err) => {
                return Response::error(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
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
        None => return Response::error(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let admin_id = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => *data.id(),
            Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => *data.admin_id(),
            Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
        },
    };

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        CollectionDao::db_select(ctx.dao().db(), path.collection_id()),
    ) {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if &admin_id != project_data.admin_id() {
        return Response::error(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if project_data.id() != collection_data.project_id() {
        return Response::error(&StatusCode::BAD_REQUEST, "Project ID does not match");
    }

    let record_data =
        match RecordDao::db_select(ctx.dao().db(), &collection_data, path.record_id()).await {
            Ok(data) => data,
            Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

    let mut record = HashMap::with_capacity(record_data.len());
    for (key, value) in record_data.data() {
        let value = match value.to_serde_json() {
            Ok(value) => value,
            Err(err) => {
                return Response::error(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
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
        None => return Response::error(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let admin_id = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => *data.id(),
            Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => *data.admin_id(),
            Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
        },
    };

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        CollectionDao::db_select(ctx.dao().db(), path.collection_id()),
    ) {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if &admin_id != project_data.admin_id() {
        return Response::error(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if project_data.id() != collection_data.project_id() {
        return Response::error(&StatusCode::BAD_REQUEST, "Project ID does not match");
    }

    for field_name in data.keys() {
        if !collection_data.schema_fields().contains_key(field_name) {
            return Response::error(
                &StatusCode::BAD_REQUEST,
                &format!("Field {field_name} is not exist in the collection"),
            );
        }
    }

    let mut record_data =
        match RecordDao::db_select(ctx.dao().db(), &collection_data, path.record_id()).await {
            Ok(data) => data,
            Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
        };
    for (field_name, field_props) in collection_data.schema_fields() {
        if let Some(value) = data.get(field_name) {
            if value.is_null() {
                if *field_props.required() {
                    return Response::error(
                        &StatusCode::BAD_REQUEST,
                        &format!("Value for {field_name} is required"),
                    );
                }
            }
            record_data.upsert(
                field_name,
                &match RecordColumnValue::from_serde_json(field_props.kind(), value) {
                    Ok(value) => value,
                    Err(err) => {
                        return Response::error(
                            &StatusCode::BAD_REQUEST,
                            &format!("Error in field {}: {}", field_name, err),
                        )
                    }
                },
            );
        }
    }

    if let Err(err) = record_data.db_update(ctx.dao().db()).await {
        return Response::error(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    let mut record = HashMap::with_capacity(record_data.len());
    for (key, value) in record_data.data() {
        let value = match value.to_serde_json() {
            Ok(value) => value,
            Err(err) => {
                return Response::error(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
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
        None => return Response::error(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let admin_id = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => *data.id(),
            Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => *data.admin_id(),
            Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
        },
    };

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        CollectionDao::db_select(ctx.dao().db(), path.collection_id()),
    ) {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if &admin_id != project_data.admin_id() {
        return Response::error(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if project_data.id() != collection_data.project_id() {
        return Response::error(&StatusCode::BAD_REQUEST, "Project ID does not match");
    }

    if let Err(err) =
        RecordDao::db_delete(ctx.dao().db(), collection_data.id(), path.record_id()).await
    {
        return Response::error(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
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
        None => return Response::error(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let admin_id = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => *data.id(),
            Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => *data.admin_id(),
            Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
        },
    };

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        CollectionDao::db_select(ctx.dao().db(), path.collection_id()),
    ) {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if &admin_id != project_data.admin_id() {
        return Response::error(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if project_data.id() != collection_data.project_id() {
        return Response::error(&StatusCode::BAD_REQUEST, "Project ID does not match");
    }

    let filter = match query_data.filter() {
        Some(filter) => {
            for f in filter {
                if !collection_data.schema_fields().contains_key(f.field()) {
                    return Response::error(
                        &StatusCode::BAD_REQUEST,
                        &format!("Field {} is not exist in the collection", f.field()),
                    );
                }
            }
            let mut filters = Vec::<RecordFilter>::with_capacity(filter.len());
            for f in filter {
                let schema_field_kind = match collection_data.schema_fields().get(f.field()) {
                    Some(field) => field.kind(),
                    None => {
                        return Response::error(
                            &StatusCode::BAD_REQUEST,
                            &format!("Field {} is not exist in the collection", f.field()),
                        );
                    }
                };
                let value = match RecordColumnValue::from_serde_json(schema_field_kind, f.value()) {
                    Ok(value) => value,
                    Err(err) => {
                        return Response::error(&StatusCode::BAD_REQUEST, &err.to_string());
                    }
                };
                filters.push(RecordFilter::new(f.field(), f.op(), &value));
            }
            filters
        }
        None => Vec::new(),
    };

    let pagination = RecordPagination::new(query_data.limit());

    let records_data =
        match RecordDao::db_select_many(ctx.dao().db(), &collection_data, &filter, &pagination)
            .await
        {
            Ok(data) => data,
            Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

    let mut records = Vec::with_capacity(records_data.len());
    for record_data in &records_data {
        let mut record = HashMap::with_capacity(record_data.len());
        for (key, value) in record_data.data() {
            let value = match value.to_serde_json() {
                Ok(value) => value,
                Err(err) => {
                    return Response::error(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
                }
            };
            record.insert(key.to_owned(), value);
        }
        records.push(record);
    }

    Response::data(
        &StatusCode::OK,
        &Some(PaginationRes::new(
            &records_data.len(),
            &records_data.len(),
            &1,
            &records_data.len(),
        )),
        &records
            .iter()
            .map(|record| RecordResJson::new(record))
            .collect::<Vec<_>>(),
    )
}
