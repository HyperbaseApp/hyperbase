use actix_header::actix_header;
use actix_web::{http::StatusCode, HttpResponse, HttpResponseBuilder};
use serde::Serialize;

pub mod admin;
pub mod auth;
pub mod collection;
pub mod project;
pub mod record;
pub mod schema_field;

#[actix_header("Authorization")]
#[derive(Debug)]
pub struct TokenReqHeader(String);

impl TokenReqHeader {
    pub fn get(&self) -> Option<&'_ str> {
        self.0.strip_prefix("Bearer ")
    }
}

impl From<String> for TokenReqHeader {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<TokenReqHeader> for String {
    fn from(s: TokenReqHeader) -> Self {
        s.0
    }
}

#[derive(Serialize)]
pub struct Response {
    error: Option<ErrorRes>,
    pagination: Option<PaginationRes>,
    data: Option<serde_json::Value>,
}

impl Response {
    pub fn data<T: Serialize>(
        status_code: StatusCode,
        pagination: Option<PaginationRes>,
        data: T,
    ) -> HttpResponse {
        let data = serde_json::to_value(data);

        match data {
            Ok(data) => HttpResponseBuilder::new(status_code).json(Self {
                error: None,
                pagination,
                data: Some(data),
            }),
            Err(err) => Self::error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string().as_str()),
        }
    }

    pub fn error(status_code: StatusCode, message: &str) -> HttpResponse {
        HttpResponseBuilder::new(status_code).json(Self {
            error: Some(ErrorRes {
                status: match status_code.canonical_reason() {
                    Some(status_code) => status_code.to_string(),
                    None => "Unknown".to_string(),
                },
                message: message.to_string(),
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

#[derive(Serialize)]
pub struct PaginationRes {
    limit: i64,
    count: i64,
    page: i64,
    total: i64,
}
