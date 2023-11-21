use actix_web::{http::StatusCode, web, HttpResponse};
use hb_dao::{
    collection::CollectionDao,
    project::ProjectDao,
    record::{RecordDao, Value},
    token::TokenDao,
};

use crate::{
    context::Context,
    v1::model::{
        record::{
            DeleteOneRecordReqPath, FindOneRecordReqPath, InsertOneRecordReqJson,
            InsertOneRecordReqPath, UpdateOneRecordReqJson, UpdateOneRecordReqPath,
        },
        Response, TokenReqHeader,
    },
};

pub fn record_api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/project/{project_id}/collection/{collection_id}/record")
            .route("", web::post().to(insert_one))
            .route("/{record_id}", web::get().to(find_one))
            .route("/{record_id}", web::patch().to(update_one))
            .route("/{delete_one}", web::delete().to(delete_one)),
    );
}

async fn insert_one(
    ctx: web::Data<Context>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<InsertOneRecordReqPath>,
    data: web::Json<InsertOneRecordReqJson>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token.jwt.decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (token_data, project_data) = match tokio::try_join!(
        TokenDao::db_select(&ctx.dao.db, token_claim.id()),
        ProjectDao::db_select(&ctx.dao.db, path.project_id()),
    ) {
        Ok(data) => data,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_data.admin_id() != project_data.admin_id() {
        return Response::error(StatusCode::FORBIDDEN, "This project does not belong to you");
    }

    let collection_data = match CollectionDao::db_select(&ctx.dao.db, path.collection_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.id() != collection_data.project_id() {
        return Response::error(StatusCode::BAD_REQUEST, "Project ID does not match");
    }

    let mut record_data = RecordDao::new(Some(&data.len()));
    for (key, value) in data.iter() {
        match collection_data.schema_fields().get(key) {
            Some(kind) => record_data.insert(
                &key,
                &match Value::from_serde_json(kind.kind(), value) {
                    Ok(value) => value,
                    Err(err) => return Response::error(StatusCode::BAD_REQUEST, &err.to_string()),
                },
            ),
            None => {
                return Response::error(
                    StatusCode::BAD_REQUEST,
                    &format!("{key} is not defined in collection"),
                )
            }
        }
    }

    let data = data.into_inner();
    for (key, value) in &data {
        println!("{} {:?}", key, value);
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
