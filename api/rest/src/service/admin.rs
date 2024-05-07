use actix_web::{http::StatusCode, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::Utc;
use hb_dao::{
    admin::AdminDao,
    change::{ChangeDao, ChangeState, ChangeTable},
};
use hb_token_jwt::claim::ClaimId;

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

    let admin_data = match token_claim.id() {
        ClaimId::Admin(id) => match AdminDao::db_select(ctx.dao().db(), id).await {
            Ok(data) => data,
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::UNAUTHORIZED,
                    &format!("Failed to get admin data: {err}"),
                )
            }
        },
        ClaimId::Token(_, _) => {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                "Must be logged in using password-based login",
            )
        }
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

    let mut admin_data = match token_claim.id() {
        ClaimId::Admin(id) => match AdminDao::db_select(ctx.dao().db(), id).await {
            Ok(data) => data,
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::UNAUTHORIZED,
                    &format!("Failed to get admin data: {err}"),
                )
            }
        },
        ClaimId::Token(_, _) => {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                "Must be logged in using password-based login",
            )
        }
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

    let change_data = ChangeDao::new(
        &ChangeTable::Admin,
        admin_data.id(),
        &ChangeState::Upsert,
        admin_data.updated_at(),
    );
    if let Err(err) = change_data.db_upsert(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    if let Some(internal_broadcast) = ctx.internal_broadcast() {
        let internal_broadcast = internal_broadcast.clone();
        tokio::spawn((|| async move {
            if let Err(err) = internal_broadcast.broadcast(&change_data).await {
                hb_log::error(
                    None,
                    &format!(
                        "[ApiRestServer] Error when broadcasting update_one admin to remote peer: {err}"
                    ),
                );
            }
        })());
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

    let admin_data = match token_claim.id() {
        ClaimId::Admin(id) => match AdminDao::db_select(ctx.dao().db(), id).await {
            Ok(data) => data,
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::UNAUTHORIZED,
                    &format!("Failed to get admin data: {err}"),
                )
            }
        },
        ClaimId::Token(_, _) => {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                "Must be logged in using password-based login",
            )
        }
    };

    let deleted_at = Utc::now();

    if let Err(err) = AdminDao::db_delete(ctx.dao().db(), admin_data.id()).await {
        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    let change_data = ChangeDao::new(
        &ChangeTable::Admin,
        admin_data.id(),
        &ChangeState::Delete,
        &deleted_at,
    );
    if let Err(err) = change_data.db_upsert(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string());
    }

    if let Some(internal_broadcast) = ctx.internal_broadcast() {
        let internal_broadcast = internal_broadcast.clone();
        tokio::spawn((|| async move {
            if let Err(err) = internal_broadcast.broadcast(&change_data).await {
                hb_log::error(
                    None,
                    &format!(
                        "[ApiRestServer] Error when broadcasting delete_one admin to remote peer: {err}"
                    ),
                );
            }
        })());
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &DeleteAdminResJson::new(admin_data.id()),
    )
}
