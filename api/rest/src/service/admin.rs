use actix_web::{web, HttpResponse, Responder};

use crate::model::admin::{
    DeleteOneAdminPath, FindOneAdminPath, InsertOneAdminJson, UpdateOneAdminJson,
    UpdateOneAdminPath,
};

pub fn admin_api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/rest/admin")
            .route("", web::post().to(insert_one))
            .route("/{admin_id}", web::get().to(find_one))
            .route("/{admin_id}", web::patch().to(update_one))
            .route("/{admin_id}", web::delete().to(delete_one)),
    );
}

async fn insert_one(admin: web::Json<InsertOneAdminJson>) -> impl Responder {
    HttpResponse::Ok().body(format!("admin insert_one {}", admin.email()))
}

async fn find_one(path: web::Path<FindOneAdminPath>) -> impl Responder {
    HttpResponse::Ok().body(format!("admin find_one {}", path.admin_id()))
}

async fn update_one(
    path: web::Path<UpdateOneAdminPath>,
    admin: web::Json<UpdateOneAdminJson>,
) -> impl Responder {
    HttpResponse::Ok().body(format!(
        "admin update_one {} {:?}",
        path.admin_id(),
        admin.email()
    ))
}

async fn delete_one(path: web::Path<DeleteOneAdminPath>) -> impl Responder {
    HttpResponse::Ok().body(format!("admin delete_one {}", path.admin_id()))
}