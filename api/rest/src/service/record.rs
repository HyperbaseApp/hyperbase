use actix_web::{http::StatusCode, web, HttpResponse};
use hb_dao::{
    admin::AdminDao, collection::CollectionDao, project::ProjectDao, record::ColumnValue,
    token::TokenDao,
};
use hb_token_jwt::kind::JwtTokenKind;

use crate::{
    context::ApiRestCtx,
    model::{
        record::{
            DeleteOneRecordReqPath, FindOneRecordReqPath, InsertOneRecordReqJson,
            InsertOneRecordReqPath, UpdateOneRecordReqJson, UpdateOneRecordReqPath,
        },
        Response, TokenReqHeader,
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
        "/project/{project_id}/collection/{collection_id}/record/{delete_one}",
        web::delete().to(delete_one),
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

    let project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if &admin_id != project_data.admin_id() {
        return Response::error(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    let collection_data = match CollectionDao::db_select(ctx.dao().db(), path.collection_id()).await
    {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

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

    let mut record_data = collection_data.to_record(&Some(data.len()));
    for (field_name, field_props) in collection_data.schema_fields().iter() {
        match data.get(field_name) {
            Some(value) => record_data.insert(
                field_name,
                &match ColumnValue::from_serde_json(field_props.kind(), value) {
                    Ok(value) => value,
                    Err(err) => {
                        return Response::error(
                            &StatusCode::BAD_REQUEST,
                            &format!("Error in field {}: {}", field_name, err),
                        )
                    }
                },
            ),
            None => match field_props.required() {
                true => {
                    return Response::error(
                        &StatusCode::BAD_REQUEST,
                        &format!("Value for {field_name} is required"),
                    )
                }
                false => record_data.insert(field_name, &ColumnValue::none(field_props.kind())),
            },
        }
    }

    if record_data.is_empty() {
        if let Err(err) = record_data.db_insert(ctx.dao().db()).await {
            return Response::error(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
        }
    }

    HttpResponse::Ok().body(format!(
        "record insert_one {} {}",
        path.project_id(),
        path.collection_id(),
    ))
}

async fn find_one(path: web::Path<FindOneRecordReqPath>) -> HttpResponse {
    HttpResponse::Ok().body(format!(
        "record find_one {} {} {}",
        path.project_id(),
        path.collection_id(),
        path.record_id()
    ))
}

async fn update_one(
    path: web::Path<UpdateOneRecordReqPath>,
    data: web::Json<UpdateOneRecordReqJson>,
) -> HttpResponse {
    let data = data.into_inner();
    for (key, value) in &data {
        println!("{} {:?}", key, value);
    }
    HttpResponse::Ok().body(format!(
        "record update_one {} {} {}",
        path.project_id(),
        path.collection_id(),
        path.record_id()
    ))
}

async fn delete_one(path: web::Path<DeleteOneRecordReqPath>) -> HttpResponse {
    HttpResponse::Ok().body(format!(
        "record delete_one {} {} {}",
        path.project_id(),
        path.collection_id(),
        path.record_id()
    ))
}
