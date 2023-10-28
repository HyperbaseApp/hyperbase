use actix_web::{http::StatusCode, web, HttpResponse};
use hb_dao::{project::ProjectDao, Db};
use hb_token_jwt::kind::JwtTokenKind;

use crate::{
    context::ApiRestContext as Context,
    v1::model::{
        project::{
            DeleteOneProjectReqPath, DeleteProjectResJson, FindOneProjectReqPath,
            InsertOneProjectReqJson, ProjectResJson, UpdateOneProjectReqJson,
            UpdateOneProjectReqPath,
        },
        Response, TokenReqHeader,
    },
};

pub fn project_api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/project")
            .route("", web::post().to(insert_one))
            .route("/{project_id}", web::get().to(find_one))
            .route("/{project_id}", web::patch().to(update_one))
            .route("/{project_id}", web::delete().to(delete_one)),
    );

    cfg.service(web::scope("/projects").route("", web::get().to(find_many)));
}

async fn insert_one(
    ctx: web::Data<Context>,
    token: web::Header<TokenReqHeader>,
    data: web::Json<InsertOneProjectReqJson>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token.jwt.decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str()),
    };

    if token_claim.kind() != &JwtTokenKind::Admin {
        return Response::error(StatusCode::BAD_REQUEST, "Must be logged in as admin");
    }

    let db = Db::ScyllaDb(&ctx.db.scylladb);

    let project_data = ProjectDao::new(token_claim.id(), data.name());

    if let Err(err) = project_data.insert(&db).await {
        return Response::error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string().as_str());
    }

    Response::data(
        StatusCode::CREATED,
        None,
        ProjectResJson::new(
            project_data.id(),
            project_data.created_at(),
            project_data.updated_at(),
            project_data.admin_id(),
            project_data.name(),
        ),
    )
}

async fn find_one(
    ctx: web::Data<Context>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<FindOneProjectReqPath>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token.jwt.decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str()),
    };

    if token_claim.kind() != &JwtTokenKind::Admin {
        return Response::error(StatusCode::BAD_REQUEST, "Must be logged in as admin");
    }

    let db = Db::ScyllaDb(&ctx.db.scylladb);

    let project_data = match ProjectDao::select(&db, path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error(StatusCode::FORBIDDEN, "This project does not belong to you");
    }

    Response::data(
        StatusCode::OK,
        None,
        ProjectResJson::new(
            project_data.id(),
            project_data.created_at(),
            project_data.updated_at(),
            project_data.admin_id(),
            project_data.name(),
        ),
    )
}

async fn update_one(
    ctx: web::Data<Context>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<UpdateOneProjectReqPath>,
    data: web::Json<UpdateOneProjectReqJson>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token.jwt.decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str()),
    };

    if token_claim.kind() != &JwtTokenKind::Admin {
        return Response::error(StatusCode::BAD_REQUEST, "Must be logged in as admin");
    }

    let db = Db::ScyllaDb(&ctx.db.scylladb);

    let mut project_data = match ProjectDao::select(&db, path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error(StatusCode::FORBIDDEN, "This project does not belong to you");
    }

    if let Some(name) = data.name() {
        project_data.set_name(name);
    }

    if data.name().is_some() {
        if let Err(err) = project_data.update(&db).await {
            return Response::error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string().as_str());
        }
    }

    Response::data(
        StatusCode::OK,
        None,
        ProjectResJson::new(
            project_data.id(),
            project_data.created_at(),
            project_data.updated_at(),
            project_data.admin_id(),
            project_data.name(),
        ),
    )
}

async fn delete_one(
    ctx: web::Data<Context>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<DeleteOneProjectReqPath>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token.jwt.decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str()),
    };

    if token_claim.kind() != &JwtTokenKind::Admin {
        return Response::error(StatusCode::BAD_REQUEST, "Must be logged in as admin");
    }

    let db = Db::ScyllaDb(&ctx.db.scylladb);

    let project_data = match ProjectDao::select(&db, path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error(StatusCode::FORBIDDEN, "This project does not belong to you");
    }

    if let Err(err) = ProjectDao::delete(&db, path.project_id()).await {
        return Response::error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string().as_str());
    }

    Response::data(
        StatusCode::OK,
        None,
        DeleteProjectResJson::new(project_data.id()),
    )
}

async fn find_many(ctx: web::Data<Context>, token: web::Header<TokenReqHeader>) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token.jwt.decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str()),
    };

    if token_claim.kind() != &JwtTokenKind::Admin {
        return Response::error(StatusCode::BAD_REQUEST, "Must be logged in as admin");
    }

    let db = Db::ScyllaDb(&ctx.db.scylladb);

    HttpResponse::Ok().body(format!("project find_many"))
}
