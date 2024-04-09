use actix_web::{http::StatusCode, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use hb_dao::{admin::AdminDao, log::LogDao};
use hb_token_jwt::kind::JwtTokenKind;

use crate::{
    context::ApiRestCtx,
    model::{
        log::{FindManyLogReqPath, FindManyLogReqQuery, LogResJson},
        PaginationRes, Response,
    },
};

pub fn log_api(cfg: &mut web::ServiceConfig) {
    cfg.route("/project/{project_id}/logs", web::get().to(find_many));
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

    if token_claim.kind() != &JwtTokenKind::Admin {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using password-based login",
        );
    }

    if let Err(err) = AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            &format!("Failed to get user data: {err}"),
        );
    }

    let (logs_data, total) = match LogDao::db_select_many_by_admin_id_and_project_id(
        ctx.dao().db(),
        token_claim.id(),
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
