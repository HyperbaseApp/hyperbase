use std::sync::Arc;

use actix_web::{web, HttpResponse, Responder};

pub fn collection_api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/rest/collection")
            .route("", web::post().to(insert_one))
            .route("/{collection_id}", web::get().to(find_one))
            .route("/{collection_id}", web::patch().to(update_one))
            .route("/{collection_id}", web::patch().to(delete_one)),
    );

    cfg.service(web::scope("/api/v1/rest/collections").route("", web::get().to(find_many)));
}

async fn insert_one() -> impl Responder {
    HttpResponse::Ok().body("collection insert_one")
}

async fn find_one(path: web::Path<(Arc<str>,)>) -> impl Responder {
    let collection_id = path.into_inner().0;
    HttpResponse::Ok().body(format!("collection find_one: {collection_id}"))
}

async fn update_one(path: web::Path<(Arc<str>,)>) -> impl Responder {
    let collection_id = path.into_inner().0;
    HttpResponse::Ok().body(format!("collection update_one: {collection_id}"))
}

async fn delete_one(path: web::Path<(Arc<str>,)>) -> impl Responder {
    let collection_id = path.into_inner().0;
    HttpResponse::Ok().body(format!("collection delete_one: {collection_id}"))
}

async fn find_many() -> impl Responder {
    HttpResponse::Ok().body("collection find_many")
}
