use actix_web::{http::StatusCode, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use hb_dao::{collection::CollectionDao, record::RecordDao};
use hb_token_jwt::kind::JwtTokenKind;

use crate::{context::ApiRestCtx, model::Response};

pub fn user_api(cfg: &mut web::ServiceConfig) {
    cfg.route("/user", web::get().to(find_one));
}

async fn find_one(ctx: web::Data<ApiRestCtx>, auth: BearerAuth) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if token_claim.kind() == &JwtTokenKind::Admin {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            "Must be logged in using token-based login",
        );
    }

    match token_claim.user() {
        Some(user) => {
            let collection_data =
                match CollectionDao::db_select(ctx.dao().db(), user.collection_id()).await {
                    Ok(data) => data,
                    Err(err) => {
                        return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string())
                    }
                };

            let user_data = match RecordDao::db_select(
                ctx.dao().db(),
                user.id(),
                &None,
                &HashSet::new(),
                &collection_data,
            )
            .await
            {
                Ok(data) => data,
                Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
            };

            let mut user = HashMap::with_capacity(user_data.len());
            for (key, value) in user_data.data() {
                let value = match value.to_serde_json() {
                    Ok(value) => value,
                    Err(err) => {
                        return Response::error_raw(
                            &StatusCode::INTERNAL_SERVER_ERROR,
                            &err.to_string(),
                        )
                    }
                };
                user.insert(key.to_owned(), value);
            }

            Response::data(&StatusCode::OK, &None, &user)
        }
        None => Response::data(
            &StatusCode::OK,
            &None,
            "User logged in using anonymous token-based login method",
        ),
    }
}
