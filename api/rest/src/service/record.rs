use actix_web::{http::StatusCode, web, HttpResponse};
use hb_dao::{collection::CollectionDao, project::ProjectDao, record::Value, token::TokenDao};

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

    let (token_data, project_data) = match tokio::try_join!(
        TokenDao::db_select(ctx.dao().db(), token_claim.id()),
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
    ) {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_data.admin_id() != project_data.admin_id() {
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

    if collection_data.schema_fields().keys().len() != data.keys().len() {
        for field_name in data.keys() {
            if !collection_data.schema_fields().contains_key(field_name) {
                return Response::error(
                    &StatusCode::BAD_REQUEST,
                    &format!("Field {field_name} is not exist in the collection"),
                );
            }
        }
    }

    let mut record_data = collection_data.to_record(&Some(data.len()));
    for (field_name, field_props) in collection_data.schema_fields().iter() {
        match data.get(field_name) {
            Some(value) => record_data.insert(
                field_name,
                &match Value::from_serde_json(field_props.kind(), value) {
                    Ok(value) => value,
                    Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
                },
            ),
            None => match field_props.required() {
                true => {
                    return Response::error(
                        &StatusCode::BAD_REQUEST,
                        &format!("value for {field_name} is required"),
                    )
                }
                false => record_data.insert(field_name, &Value::none(field_props.kind())),
            },
        }
    }

    HttpResponse::Ok().body(format!(
        "record insert_one {} {}",
        path.project_id(),
        path.collection_id()
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
