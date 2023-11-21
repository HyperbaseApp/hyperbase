use actix_web::{
    body::{to_bytes, MessageBody},
    dev::ServiceResponse,
    http::header,
    middleware::ErrorHandlerResponse,
    Result,
};
use futures::executor;

use crate::v1::model::Response;

pub fn default_error_handler<B: MessageBody>(
    svc_res: ServiceResponse<B>,
) -> Result<ErrorHandlerResponse<B>> {
    if let Some(content_type) = svc_res.response().headers().get(header::CONTENT_TYPE) {
        if let Ok(content_type) = content_type.to_str() {
            if content_type.to_lowercase() == "application/json" {
                return Ok(ErrorHandlerResponse::Response(svc_res.map_into_left_body()));
            }
        }
    }

    let (req, res) = svc_res.into_parts();

    let status_code = res.status();
    let body = executor::block_on(async {
        match to_bytes(res.into_body()).await {
            Ok(bytes) => match String::from_utf8(bytes.to_vec()) {
                Ok(str) => str,
                Err(err) => err.to_string(),
            },
            Err(err) => err.into().to_string(),
        }
    });

    let res = Response::error(status_code, &body);

    Ok(ErrorHandlerResponse::Response(
        ServiceResponse::new(req, res).map_into_right_body(),
    ))
}
