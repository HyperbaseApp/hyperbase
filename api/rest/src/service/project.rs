use actix_web::{http::StatusCode, web, HttpResponse};
use hb_dao::project::ProjectDao;
use hb_token_jwt::kind::JwtTokenKind;

use crate::{
    context::ApiRestCtx,
    model::{
        project::{
            DeleteOneProjectReqPath, DeleteProjectResJson, FindOneProjectReqPath,
            InsertOneProjectReqJson, ProjectResJson, UpdateOneProjectReqJson,
            UpdateOneProjectReqPath,
        },
        PaginationRes, Response, TokenReqHeader,
    },
};

pub fn project_api(cfg: &mut web::ServiceConfig) {
    cfg.route("/project", web::post().to(insert_one))
        .route("/project/{project_id}", web::get().to(find_one))
        .route("/project/{project_id}", web::patch().to(update_one))
        .route("/project/{project_id}", web::delete().to(delete_one))
        .route("/projects", web::get().to(find_many));
}

async fn insert_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    data: web::Json<InsertOneProjectReqJson>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    let project_data = ProjectDao::new(token_claim.id(), data.name());

    if let Err(err) = project_data.db_insert(ctx.dao().db()).await {
        return Response::error(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::CREATED,
        &None,
        &ProjectResJson::new(
            project_data.id(),
            project_data.created_at(),
            project_data.updated_at(),
            project_data.name(),
        ),
    )
}

async fn find_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<FindOneProjectReqPath>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    let project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &ProjectResJson::new(
            project_data.id(),
            project_data.created_at(),
            project_data.updated_at(),
            project_data.name(),
        ),
    )
}

async fn update_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<UpdateOneProjectReqPath>,
    data: web::Json<UpdateOneProjectReqJson>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    let mut project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if let Some(name) = data.name() {
        project_data.set_name(name);
    }

    if !data.is_all_none() {
        if let Err(err) = project_data.db_update(ctx.dao().db()).await {
            return Response::error(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
        }
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &ProjectResJson::new(
            project_data.id(),
            project_data.created_at(),
            project_data.updated_at(),
            project_data.name(),
        ),
    )
}

async fn delete_one(
    ctx: web::Data<ApiRestCtx>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<DeleteOneProjectReqPath>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    let project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != token_claim.id() {
        return Response::error(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if let Err(err) = ProjectDao::db_delete(ctx.dao().db(), path.project_id()).await {
        return Response::error(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &DeleteProjectResJson::new(project_data.id()),
    )
}

async fn find_many(ctx: web::Data<ApiRestCtx>, token: web::Header<TokenReqHeader>) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(&StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::User {
        return Response::error(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    let projects_data =
        match ProjectDao::db_select_many_by_admin_id(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => data,
            Err(err) => return Response::error(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

    Response::data(
        &StatusCode::OK,
        &Some(PaginationRes::new(
            &projects_data.len(),
            &projects_data.len(),
            &1,
            &projects_data.len(),
        )),
        &projects_data
            .iter()
            .map(|data| {
                ProjectResJson::new(data.id(), data.created_at(), data.updated_at(), data.name())
            })
            .collect::<Vec<_>>(),
    )
}
