use actix_web::{web, HttpResponse, Responder};

use crate::v1::model::project::{
    DeleteOneProjectReqPath, FindManyProjectReqPath, FindOneProjectReqPath, InsertOneProjectReqJson,
    InsertOneProjectReqPath, UpdateOneProjectReqJson, UpdateOneProjectReqPath,
};

pub fn project_api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/{admin_id}/project")
            .route("", web::post().to(insert_one))
            .route("/{project_id}", web::get().to(find_one))
            .route("/{project_id}", web::patch().to(update_one))
            .route("/{project_id}", web::delete().to(delete_one)),
    );

    cfg.service(
        web::scope("/api/v1/rest/admin/{admin_id}/projects").route("", web::get().to(find_many)),
    );
}

async fn insert_one(
    path: web::Path<InsertOneProjectReqPath>,
    data: web::Json<InsertOneProjectReqJson>,
) -> impl Responder {
    HttpResponse::Ok().body(format!(
        "project insert_one {} {}",
        path.admin_id(),
        data.name()
    ))
}

async fn find_one(path: web::Path<FindOneProjectReqPath>) -> impl Responder {
    HttpResponse::Ok().body(format!(
        "project find_one {} {}",
        path.admin_id(),
        path.project_id()
    ))
}

async fn update_one(
    path: web::Path<UpdateOneProjectReqPath>,
    data: web::Json<UpdateOneProjectReqJson>,
) -> impl Responder {
    HttpResponse::Ok().body(format!(
        "project update_one {} {} {:?}",
        path.admin_id(),
        path.project_id(),
        data.name()
    ))
}

async fn delete_one(path: web::Path<DeleteOneProjectReqPath>) -> impl Responder {
    HttpResponse::Ok().body(format!(
        "project delete_one {} {}",
        path.admin_id(),
        path.project_id(),
    ))
}

async fn find_many(path: web::Path<FindManyProjectReqPath>) -> impl Responder {
    HttpResponse::Ok().body(format!("project find_many {}", path.admin_id()))
}
