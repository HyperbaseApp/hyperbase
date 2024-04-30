use actix_web::{http::StatusCode, web, HttpRequest, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use hb_api_websocket::{message::Target, session::UserSession};
use hb_dao::{admin::AdminDao, log::LogDao, project::ProjectDao};
use hb_token_jwt::claim::ClaimId;

use crate::{
    context::ApiRestCtx,
    model::{
        log::{
            FindManyLogReqPath, FindManyLogReqQuery, LogResJson, SubscribeLogReqPath,
            SubscribeLogReqQuery,
        },
        PaginationRes, Response,
    },
};

pub fn log_api(cfg: &mut web::ServiceConfig) {
    cfg.route("/project/{project_id}/logs", web::get().to(find_many))
        .route(
            "/project/{project_id}/logs/subscribe",
            web::get().to(subscribe),
        );
}

async fn find_many(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<FindManyLogReqPath>,
    query: web::Query<FindManyLogReqQuery>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let admin_id = match token_claim.id() {
        ClaimId::Admin(id) => match AdminDao::db_select(ctx.dao().db(), id).await {
            Ok(data) => *data.id(),
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

    let project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != &admin_id {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    let (logs_data, total) = match LogDao::db_select_many_by_admin_id_and_project_id(
        ctx.dao().db(),
        &admin_id,
        path.project_id(),
        query.before_id(),
        query.limit(),
    )
    .await
    {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let total = match usize::try_from(total) {
        Ok(data) => data,
        Err(err) => {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
        }
    };

    Response::data(
        &StatusCode::OK,
        &Some(PaginationRes::new(&logs_data.len(), &total)),
        &logs_data
            .iter()
            .map(|data| {
                LogResJson::new(
                    data.id(),
                    data.created_at(),
                    data.kind().to_str(),
                    data.message(),
                )
            })
            .collect::<Vec<_>>(),
    )
}

async fn subscribe(
    ctx: web::Data<ApiRestCtx>,
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<SubscribeLogReqPath>,
    query: web::Query<SubscribeLogReqQuery>,
) -> HttpResponse {
    let token = query.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let admin_id = match token_claim.id() {
        ClaimId::Admin(id) => match AdminDao::db_select(ctx.dao().db(), id).await {
            Ok(data) => *data.id(),
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

    let project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if project_data.admin_id() != &admin_id {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    let (res, session, msg_stream) = match actix_ws_ng::handle(&req, stream) {
        Ok(res) => res,
        Err(err) => {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
        }
    };

    tokio::task::spawn_local((|| async move {
        let _ = ctx
            .websocket()
            .handler()
            .clone()
            .connection(
                UserSession::Admin(admin_id),
                Target::Log,
                session,
                msg_stream,
            )
            .await;
    })());

    res
}
