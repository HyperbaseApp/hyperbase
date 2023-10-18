use actix_web::{web, HttpResponse, Responder};

use crate::v1::model::record::{
    DeleteOneRecordPath, FindOneRecordPath, InsertOneRecordJson, InsertOneRecordPath,
    UpdateOneRecordJson, UpdateOneRecordPath,
};

pub fn record_api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/collection/{collection_id}/record")
            .route("", web::post().to(insert_one))
            .route("/{record_id}", web::get().to(find_one))
            .route("/{record_id}", web::patch().to(update_one))
            .route("/{delete_one}", web::delete().to(delete_one)),
    );
}

async fn insert_one(
    path: web::Path<InsertOneRecordPath>,
    data: web::Json<InsertOneRecordJson>,
) -> impl Responder {
    let data = data.into_inner();
    for (key, value) in &data {
        println!("{} {:?}", key, value);
    }
    HttpResponse::Ok().body(format!("record insert_one {}", path.collection_id()))
}

async fn find_one(path: web::Path<FindOneRecordPath>) -> impl Responder {
    HttpResponse::Ok().body(format!(
        "record find_one {} {}",
        path.collection_id(),
        path.record_id()
    ))
}

async fn update_one(
    path: web::Path<UpdateOneRecordPath>,
    data: web::Json<UpdateOneRecordJson>,
) -> impl Responder {
    let data = data.into_inner();
    for (key, value) in &data {
        println!("{} {:?}", key, value);
    }
    HttpResponse::Ok().body(format!(
        "record update_one {} {}",
        path.collection_id(),
        path.record_id()
    ))
}

async fn delete_one(path: web::Path<DeleteOneRecordPath>) -> impl Responder {
    HttpResponse::Ok().body(format!(
        "record delete_one {} {}",
        path.collection_id(),
        path.record_id()
    ))
}
