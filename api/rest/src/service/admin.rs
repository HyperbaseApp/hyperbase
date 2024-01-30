use actix_web::{http::StatusCode, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use hb_dao::admin::AdminDao;
use hb_token_jwt::kind::JwtTokenKind;

use crate::{
    context::ApiRestCtx,
    model::{
        admin::{AdminResJson, DeleteAdminResJson, UpdateOneAdminReqJson},
        Response,
    },
};

pub fn admin_api(cfg: &mut web::ServiceConfig) {
    cfg.route("/admin", web::get().to(find_one))
        .route("/admin", web::patch().to(update_one))
        .route("/admin", web::delete().to(delete_one));
}

async fn find_one(ctx: web::Data<ApiRestCtx>, auth: BearerAuth) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::Admin {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    let admin_data = match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    Response::data(
        &StatusCode::OK,
        &None,
        &AdminResJson::new(
            admin_data.id(),
            admin_data.created_at(),
            admin_data.updated_at(),
            admin_data.email(),
        ),
    )
}

async fn update_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    data: web::Json<UpdateOneAdminReqJson>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::Admin {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    let mut admin_data = match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if data.is_all_none() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "No request fields to be updated");
    }

    if let Some(password) = data.password() {
        let password_hash = match ctx.hash().argon2().hash_password(password.as_bytes()) {
            Ok(hash) => hash,
            Err(err) => {
                return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
            }
        };

        admin_data.set_password_hash(&password_hash.to_string());
    }

    if let Err(err) = admin_data.db_update(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &AdminResJson::new(
            admin_data.id(),
            admin_data.created_at(),
            admin_data.updated_at(),
            admin_data.email(),
        ),
    )
}

async fn delete_one(ctx: web::Data<ApiRestCtx>, auth: BearerAuth) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() != &JwtTokenKind::Admin {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    if let Err(err) = AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    if let Err(err) = AdminDao::db_delete(ctx.dao().db(), token_claim.id()).await {
        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &DeleteAdminResJson::new(token_claim.id()),
    )
}
