use actix_multipart::form::MultipartForm;
use actix_web::{http::StatusCode, web, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use hb_dao::{
    admin::AdminDao, bucket::BucketDao, file::FileDao, project::ProjectDao, token::TokenDao,
};
use hb_token_jwt::kind::JwtTokenKind;
use tokio::fs;
use uuid::Uuid;

use crate::{
    context::ApiRestCtx,
    model::{
        file::{
            DeleteFileResJson, DeleteOneFileReqPath, FileResJson, FindManyFileReqPath,
            FindOneFileReqPath, InsertOneFileReqForm, InsertOneFileReqPath, UpdateOneFileReqJson,
            UpdateOneFileReqPath,
        },
        PaginationRes, Response,
    },
};

pub fn file_api(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/project/{project_id}/bucket/{bucket_id}/file",
        web::post().to(insert_one),
    )
    .route(
        "/project/{project_id}/bucket/{bucket_id}/file/{file_id}",
        web::get().to(find_one),
    )
    .route(
        "/project/{project_id}/bucket/{bucket_id}/file/{file_id}",
        web::patch().to(update_one),
    )
    .route(
        "/project/{project_id}/bucket/{bucket_id}/file/{file_id}",
        web::delete().to(delete_one),
    )
    .route(
        "/project/{project_id}/bucket/{bucket_id}/files",
        web::post().to(find_many),
    );
}

async fn insert_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<InsertOneFileReqPath>,
    form: MultipartForm<InsertOneFileReqForm>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.admin_id(), Some(data)),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get token data: {err}"),
                )
            }
        },
    };

    if let Some(token_data) = &token_data {
        if !token_data.is_allow_insert_file(path.bucket_id()) {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to write data to this bucket",
            );
        }
    }

    let (project_data, bucket_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        BucketDao::db_select(ctx.dao().db(), path.bucket_id())
    ) {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if &admin_id != project_data.admin_id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if project_data.id() != bucket_data.project_id() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Project id does not match");
    }

    let mut file_name = Uuid::now_v7().to_string();
    if let Some(name) = form.file_name() {
        file_name = name;
    }
    let mut content_type = mime::APPLICATION_OCTET_STREAM;
    if let Some(mime) = form.content_type() {
        content_type = mime.clone();
    }
    let size = match i64::try_from(*form.size()) {
        Ok(size) => size,
        Err(err) => {
            return Response::error_raw(
                &StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to save file to the bucket: {}", err),
            )
        }
    };
    let file_data = FileDao::new(path.bucket_id(), &file_name, &content_type, &size);

    if let Err(err) = fs::copy(
        form.file_path(),
        &format!("{}/{}", ctx.bucket_path(), file_data.id()),
    )
    .await
    {
        return Response::error_raw(
            &StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to save file to the bucket: {}", err),
        );
    }

    if let Err(err) = file_data.db_insert(ctx.dao().db()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::CREATED,
        &None,
        &FileResJson::new(
            file_data.id(),
            file_data.created_at(),
            file_data.updated_at(),
            file_data.bucket_id(),
            file_data.file_name(),
            &file_data.content_type().to_string(),
            file_data.size(),
        ),
    )
}

async fn find_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<FindOneFileReqPath>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.admin_id(), Some(data)),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get token data: {err}"),
                )
            }
        },
    };

    if let Some(token_data) = &token_data {
        if !token_data.is_allow_find_one_file(path.bucket_id()) {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to read this bucket",
            );
        }
    }

    let (project_data, bucket_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        BucketDao::db_select(ctx.dao().db(), path.bucket_id())
    ) {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if &admin_id != project_data.admin_id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if project_data.id() != bucket_data.project_id() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Project id does not match");
    }

    let file_data = match FileDao::db_select(ctx.dao().db(), path.file_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if file_data.bucket_id() != bucket_data.id() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Bucket id does not match");
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &FileResJson::new(
            file_data.id(),
            file_data.created_at(),
            file_data.updated_at(),
            file_data.bucket_id(),
            file_data.file_name(),
            &file_data.content_type().to_string(),
            file_data.size(),
        ),
    )
}

async fn update_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<UpdateOneFileReqPath>,
    data: web::Json<UpdateOneFileReqJson>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.admin_id(), Some(data)),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get token data: {err}"),
                )
            }
        },
    };

    if let Some(token_data) = &token_data {
        if !token_data.is_allow_update_file(path.bucket_id()) {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to update this file",
            );
        }
    }

    let (project_data, bucket_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        BucketDao::db_select(ctx.dao().db(), path.bucket_id())
    ) {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if &admin_id != project_data.admin_id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if project_data.id() != bucket_data.project_id() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Project id does not match");
    }

    let mut file_data = match FileDao::db_select(ctx.dao().db(), path.file_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if file_data.bucket_id() != bucket_data.id() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Bucket id does not match");
    }

    if let Some(file_name) = data.file_name() {
        file_data.set_file_name(file_name);
    }

    if !data.is_all_none() {
        if let Err(err) = file_data.db_update(ctx.dao().db()).await {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
        }
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &FileResJson::new(
            file_data.id(),
            file_data.created_at(),
            file_data.updated_at(),
            file_data.bucket_id(),
            file_data.file_name(),
            &file_data.content_type().to_string(),
            file_data.size(),
        ),
    )
}

async fn delete_one(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<DeleteOneFileReqPath>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.admin_id(), Some(data)),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get token data: {err}"),
                )
            }
        },
    };

    if let Some(token_data) = &token_data {
        if !token_data.is_allow_delete_file(path.bucket_id()) {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to delete this file",
            );
        }
    }

    let (project_data, bucket_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        BucketDao::db_select(ctx.dao().db(), path.bucket_id())
    ) {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if &admin_id != project_data.admin_id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if project_data.id() != bucket_data.project_id() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Project id does not match");
    }

    let file_data = match FileDao::db_select(ctx.dao().db(), path.file_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if file_data.bucket_id() != bucket_data.id() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Bucket id does not match");
    }

    if let Err(err) = fs::remove_file(&format!("{}/{}", ctx.bucket_path(), file_data.id())).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    if let Err(err) = FileDao::db_delete(ctx.dao().db(), path.file_id()).await {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::OK,
        &None,
        &DeleteFileResJson::new(path.file_id()),
    )
}

async fn find_many(
    ctx: web::Data<ApiRestCtx>,
    auth: BearerAuth,
    path: web::Path<FindManyFileReqPath>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data) = match token_claim.kind() {
        JwtTokenKind::User => match AdminDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.id(), None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get user data: {err}"),
                )
            }
        },
        JwtTokenKind::Token => match TokenDao::db_select(ctx.dao().db(), token_claim.id()).await {
            Ok(data) => (*data.admin_id(), Some(data)),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    &format!("Failed to get token data: {err}"),
                )
            }
        },
    };

    if let Some(token_data) = &token_data {
        if !token_data.is_allow_find_many_files(path.bucket_id()) {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to read these files",
            );
        }
    }

    let (project_data, bucket_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), path.project_id()),
        BucketDao::db_select(ctx.dao().db(), path.bucket_id())
    ) {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if &admin_id != project_data.admin_id() {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "This project does not belong to you",
        );
    }

    if project_data.id() != bucket_data.project_id() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Project id does not match");
    }

    let files_data =
        match FileDao::db_select_many_by_bucket_id(ctx.dao().db(), path.bucket_id()).await {
            Ok(data) => data,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

    Response::data(
        &StatusCode::OK,
        &Some(PaginationRes::new(&files_data.len(), &files_data.len())),
        &files_data
            .iter()
            .map(|data| {
                FileResJson::new(
                    data.id(),
                    data.created_at(),
                    data.updated_at(),
                    data.bucket_id(),
                    data.file_name(),
                    &data.content_type().to_string(),
                    data.size(),
                )
            })
            .collect::<Vec<_>>(),
    )
}
