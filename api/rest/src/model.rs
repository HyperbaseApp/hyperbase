use actix_web::{http::StatusCode, HttpResponse, HttpResponseBuilder};
use hb_error::Error;
use serde::Serialize;

pub mod admin;
pub mod auth;
pub mod bucket;
pub mod collection;
pub mod file;
pub mod project;
pub mod record;
pub mod token;

#[derive(Serialize)]
pub struct Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ErrorRes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pagination: Option<PaginationRes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

impl Response {
    pub fn data<T: Serialize>(
        status_code: &StatusCode,
        pagination: &Option<PaginationRes>,
        data: T,
    ) -> HttpResponse {
        match serde_json::to_value(data) {
            Ok(data) => HttpResponseBuilder::new(*status_code).json(Self {
                error: None,
                pagination: *pagination,
                data: Some(data),
            }),
            Err(err) => {
                hb_log::error(None, &err);
                Self::error(&Error::InternalServerError(err.to_string()))
            }
        }
    }

    pub fn error(err: &Error) -> HttpResponse {
        let (status_code, message) = match err {
            Error::BadRequest(msg) => (&StatusCode::BAD_REQUEST, msg),
            Error::Forbidden(msg) => (&StatusCode::FORBIDDEN, msg),
            Error::InternalServerError(msg) => (&StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        Self::error_raw(status_code, message)
    }

    pub fn error_raw(status_code: &StatusCode, message: &str) -> HttpResponse {
        hb_log::error(None, message);

        HttpResponseBuilder::new(*status_code).json(Self {
            error: Some(ErrorRes {
                status: match status_code.canonical_reason() {
                    Some(status_code) => status_code.to_owned(),
                    None => "Unknown".to_owned(),
                },
                message: message.to_owned(),
            }),
            pagination: None,
            data: None,
        })
    }
}

#[derive(Serialize)]
pub struct ErrorRes {
    status: String,
    message: String,
}

#[derive(Serialize, Clone, Copy)]
pub struct PaginationRes {
    count: usize,
    total: usize,
}

impl PaginationRes {
    pub fn new(count: &usize, total: &usize) -> Self {
        Self {
            count: *count,
            total: *total,
        }
    }
}
