use actix_web::{web, HttpResponse, Responder};

use crate::v1::model::collection::{
    DeleteOneCollectionReqPath, FindOneCollectionReqPath, InsertOneCollectionReqJson,
    InsertOneCollectionReqPath, UpdateOneCollectionReqPath,
};

pub fn collection_api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/{admin_id}/project/{project_id}/collection")
            .route("", web::post().to(insert_one))
            .route("/{collection_id}", web::get().to(find_one))
            .route("/{collection_id}", web::patch().to(update_one))
            .route("/{collection_id}", web::patch().to(delete_one)),
    );

    cfg.service(web::scope("/api/v1/rest/collections").route("", web::get().to(find_many)));
}

async fn insert_one(
    path: web::Path<InsertOneCollectionReqPath>,
    data: web::Json<InsertOneCollectionReqJson>,
) -> impl Responder {
    HttpResponse::Ok().body(format!(
        "collection insert_one {} {} {}",
        path.admin_id(),
        path.project_id(),
        data.name()
    ))
}

async fn find_one(path: web::Path<FindOneCollectionReqPath>) -> impl Responder {
    HttpResponse::Ok().body(format!("collection find_one: {}", path.collection_id()))
}

async fn update_one(
    path: web::Path<UpdateOneCollectionReqPath>,
    data: web::Json<InsertOneCollectionReqJson>,
) -> impl Responder {
    HttpResponse::Ok().body(format!(
        "collection update_one: {}, {}",
        path.collection_id(),
        data.name()
    ))
}

async fn delete_one(path: web::Path<DeleteOneCollectionReqPath>) -> impl Responder {
    HttpResponse::Ok().body(format!("collection delete_one: {}", path.collection_id()))
}

async fn find_many() -> impl Responder {
    HttpResponse::Ok().body("collection find_many")
}
