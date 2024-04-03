use actix_web::{http::StatusCode, web, HttpResponse};
use hb_dao::value::ColumnKind;
use strum::IntoEnumIterator;

use crate::{context::ApiRestCtx, model::Response};

pub fn info_api(cfg: &mut web::ServiceConfig) {
    cfg.route("/info/schema_fields", web::get().to(schema_fields))
        .route(
            "/info/admin_registration",
            web::get().to(admin_registration),
        );
}

async fn schema_fields() -> HttpResponse {
    let mut fields = Vec::new();

    for kind in ColumnKind::iter() {
        fields.push(kind.to_str().to_owned());
    }

    Response::data(&StatusCode::OK, &None, &fields)
}

async fn admin_registration(ctx: web::Data<ApiRestCtx>) -> HttpResponse {
    Response::data(&StatusCode::OK, &None, ctx.admin_registration())
}
