use actix_files::NamedFile;
use actix_multipart::form::MultipartForm;
use actix_web::{
    http::{
        header::{self, HeaderValue},
        StatusCode,
    },
    web, HttpRequest, HttpResponse,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use ahash::HashSet;
use hb_dao::{
    admin::AdminDao, bucket::BucketDao, bucket_rule::BucketPermission, collection::CollectionDao,
    file::FileDao, project::ProjectDao, record::RecordDao, token::TokenDao, value::ColumnValue,
};
use hb_token_jwt::claim::ClaimId;
use uuid::Uuid;

use crate::{
    context::ApiRestCtx,
    model::{
        file::{
            DeleteFileResJson, DeleteOneFileReqPath, FileResJson, FindManyFileReqPath,
            FindManyFileReqQuery, FindOneFileReqPath, FindOneFileReqQuery, HeadFindOneFileReqPath,
            HeadFindOneFileReqQuery, InsertOneFileReqForm, InsertOneFileReqPath,
            UpdateOneFileReqJson, UpdateOneFileReqPath,
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
        web::head().to(head_find_one),
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
        web::get().to(find_many),
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

    let (admin_id, token_data, user_claim) = match token_claim.id() {
        ClaimId::Admin(id) => match AdminDao::db_select(ctx.dao().db(), id).await {
            Ok(data) => (*data.id(), None, None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::UNAUTHORIZED,
                    &format!("Failed to get admin data: {err}"),
                )
            }
        },
        ClaimId::Token(token_id, user_claim) => {
            match TokenDao::db_select(ctx.dao().db(), token_id).await {
                Ok(data) => (*data.admin_id(), Some(data), *user_claim),
                Err(err) => {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        &format!("Failed to get token data: {err}"),
                    )
                }
            }
        }
    };

    if let Some(token_data) = &token_data {
        if !token_data
            .is_allow_insert_file(ctx.dao().db(), path.bucket_id())
            .await
        {
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

    let created_by = if matches!(token_claim.id(), ClaimId::Admin(_)) {
        admin_id
    } else if let Some(user_claim) = user_claim {
        let collection_data =
            match CollectionDao::db_select(ctx.dao().db(), user_claim.collection_id()).await {
                Ok(data) => data,
                Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
            };
        let user_data = match RecordDao::db_select(
            ctx.dao().db(),
            user_claim.id(),
            &None,
            &HashSet::from_iter(["_id"]),
            &collection_data,
            &token_data.is_none(),
        )
        .await
        {
            Ok(data) => data,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

        let mut user_id = None;
        if let Some(id) = user_data.get("_id") {
            if let ColumnValue::Uuid(id) = id {
                if let Some(id) = id {
                    user_id = Some(*id);
                }
            }
        }

        if let Some(user_id) = user_id {
            user_id
        } else {
            return Response::error_raw(&StatusCode::BAD_REQUEST, "User doesn't found");
        }
    } else if let Some(token_data) = token_data {
        *token_data.id()
    } else {
        return Response::error_raw(
            &StatusCode::INTERNAL_SERVER_ERROR,
            "Cannot determine created_by",
        );
    };
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
                &format!("Failed to save file to the bucket: {err}"),
            )
        }
    };
    let file_data = FileDao::new(
        &created_by,
        path.bucket_id(),
        &file_name,
        &content_type,
        &size,
        &form.public(),
    );

    if let Err(err) = file_data
        .save(ctx.dao().db(), bucket_data.path(), form.file_path())
        .await
    {
        return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string());
    }

    Response::data(
        &StatusCode::CREATED,
        &None,
        &FileResJson::new(
            file_data.id(),
            file_data.created_by(),
            file_data.created_at(),
            file_data.updated_at(),
            file_data.bucket_id(),
            file_data.file_name(),
            &file_data.content_type().to_string(),
            file_data.size(),
            file_data.public(),
        ),
    )
}

async fn head_find_one(
    ctx: web::Data<ApiRestCtx>,
    req: HttpRequest,
    path: web::Path<HeadFindOneFileReqPath>,
    query: web::Query<HeadFindOneFileReqQuery>,
) -> HttpResponse {
    let bucket_data = match BucketDao::db_select(ctx.dao().db(), path.bucket_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let file_data = match FileDao::db_select(ctx.dao().db(), &bucket_data, path.file_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if !*file_data.public() {
        if let Some(token) = query.token() {
            let token_claim = match ctx.token().jwt().decode(token) {
                Ok(token) => token,
                Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
            };

            let (admin_id, token_data, user_claim) = match token_claim.id() {
                ClaimId::Admin(id) => match AdminDao::db_select(ctx.dao().db(), id).await {
                    Ok(data) => (*data.id(), None, None),
                    Err(err) => {
                        return Response::error_raw(
                            &StatusCode::UNAUTHORIZED,
                            &format!("Failed to get admin data: {err}"),
                        )
                    }
                },
                ClaimId::Token(token_id, user_claim) => {
                    match TokenDao::db_select(ctx.dao().db(), token_id).await {
                        Ok(data) => (*data.admin_id(), Some(data), *user_claim),
                        Err(err) => {
                            return Response::error_raw(
                                &StatusCode::BAD_REQUEST,
                                &format!("Failed to get token data: {err}"),
                            )
                        }
                    }
                }
            };

            let rule_find_one = if let Some(token_data) = &token_data {
                if let Some(rule) = token_data
                    .is_allow_find_one_file(ctx.dao().db(), path.bucket_id())
                    .await
                {
                    Some(rule)
                } else {
                    return Response::error_raw(
                        &StatusCode::FORBIDDEN,
                        "This token doesn't have permission to read this file",
                    );
                }
            } else {
                None
            };

            let project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await
            {
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

            let created_by = if matches!(token_claim.id(), ClaimId::Admin(_)) {
                None
            } else if let Some(rule) = rule_find_one {
                match rule {
                    BucketPermission::All => None,
                    BucketPermission::SelfMade => match user_claim {
                        Some(user_claim) => {
                            let collection_data = match CollectionDao::db_select(
                                ctx.dao().db(),
                                user_claim.collection_id(),
                            )
                            .await
                            {
                                Ok(data) => data,
                                Err(err) => {
                                    return Response::error_raw(
                                        &StatusCode::BAD_REQUEST,
                                        &err.to_string(),
                                    )
                                }
                            };
                            let user_data = match RecordDao::db_select(
                                ctx.dao().db(),
                                user_claim.id(),
                                &None,
                                &HashSet::from_iter(["_id"]),
                                &collection_data,
                                &token_data.is_none(),
                            )
                            .await
                            {
                                Ok(data) => data,
                                Err(err) => {
                                    return Response::error_raw(
                                        &StatusCode::BAD_REQUEST,
                                        &err.to_string(),
                                    )
                                }
                            };

                            let mut user_id = None;
                            if let Some(id) = user_data.get("_id") {
                                if let ColumnValue::Uuid(id) = id {
                                    if let Some(id) = id {
                                        user_id = Some(*id);
                                    }
                                }
                            }

                            if user_id.is_none() {
                                return Response::error_raw(
                                    &StatusCode::BAD_REQUEST,
                                    "User not found",
                                );
                            }

                            user_id
                        }
                        None => {
                            if let Some(token_data) = token_data {
                                Some(*token_data.id())
                            } else {
                                return Response::error_raw(
                                    &StatusCode::INTERNAL_SERVER_ERROR,
                                    "Cannot determine created_by",
                                );
                            }
                        }
                    },
                    BucketPermission::None => {
                        return Response::error_raw(
                            &StatusCode::BAD_REQUEST,
                            "User doesn't have permission to read this file",
                        )
                    }
                }
            } else {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    "User doesn't have permission to read this file",
                );
            };

            if let Some(created_by) = &created_by {
                if created_by != file_data.created_by() {
                    return Response::error_raw(
                        &StatusCode::FORBIDDEN,
                        "User doesn't have permission to read this file",
                    );
                }
            }

            if file_data.bucket_id() != bucket_data.id() {
                return Response::error_raw(&StatusCode::BAD_REQUEST, "Bucket id does not match");
            }
        } else {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                &format!("The file is not public so the request must contain a token."),
            );
        }
    }

    let file =
        match NamedFile::open_async(&format!("{}/{}", bucket_data.path(), file_data.id())).await {
            Ok(file) => file,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

    let mut res = HttpResponse::Ok();
    for header in file.into_response(&req).headers() {
        if header.0 != header::CONTENT_DISPOSITION {
            res.append_header(header);
        }
    }
    res.insert_header((
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachment; filename=\"{}\"",
            file_data.file_name()
        ))
        .unwrap(),
    ));
    res.finish()
}

async fn find_one(
    ctx: web::Data<ApiRestCtx>,
    req: HttpRequest,
    path: web::Path<FindOneFileReqPath>,
    query: web::Query<FindOneFileReqQuery>,
) -> HttpResponse {
    let bucket_data = match BucketDao::db_select(ctx.dao().db(), path.bucket_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let file_data = match FileDao::db_select(ctx.dao().db(), &bucket_data, path.file_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if !*file_data.public() {
        if let Some(token) = query.token() {
            let token_claim = match ctx.token().jwt().decode(token) {
                Ok(token) => token,
                Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
            };

            let (admin_id, token_data, user_claim) = match token_claim.id() {
                ClaimId::Admin(id) => match AdminDao::db_select(ctx.dao().db(), id).await {
                    Ok(data) => (*data.id(), None, None),
                    Err(err) => {
                        return Response::error_raw(
                            &StatusCode::UNAUTHORIZED,
                            &format!("Failed to get admin data: {err}"),
                        )
                    }
                },
                ClaimId::Token(token_id, user_claim) => {
                    match TokenDao::db_select(ctx.dao().db(), token_id).await {
                        Ok(data) => (*data.admin_id(), Some(data), *user_claim),
                        Err(err) => {
                            return Response::error_raw(
                                &StatusCode::BAD_REQUEST,
                                &format!("Failed to get token data: {err}"),
                            )
                        }
                    }
                }
            };

            let rule_find_one = if let Some(token_data) = &token_data {
                if let Some(rule) = token_data
                    .is_allow_find_one_file(ctx.dao().db(), path.bucket_id())
                    .await
                {
                    Some(rule)
                } else {
                    return Response::error_raw(
                        &StatusCode::FORBIDDEN,
                        "This token doesn't have permission to read this file",
                    );
                }
            } else {
                None
            };

            let project_data = match ProjectDao::db_select(ctx.dao().db(), path.project_id()).await
            {
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

            let created_by = if matches!(token_claim.id(), ClaimId::Admin(_)) {
                None
            } else if let Some(rule) = rule_find_one {
                match rule {
                    BucketPermission::All => None,
                    BucketPermission::SelfMade => match user_claim {
                        Some(user_claim) => {
                            let collection_data = match CollectionDao::db_select(
                                ctx.dao().db(),
                                user_claim.collection_id(),
                            )
                            .await
                            {
                                Ok(data) => data,
                                Err(err) => {
                                    return Response::error_raw(
                                        &StatusCode::BAD_REQUEST,
                                        &err.to_string(),
                                    )
                                }
                            };
                            let user_data = match RecordDao::db_select(
                                ctx.dao().db(),
                                user_claim.id(),
                                &None,
                                &HashSet::from_iter(["_id"]),
                                &collection_data,
                                &token_data.is_none(),
                            )
                            .await
                            {
                                Ok(data) => data,
                                Err(err) => {
                                    return Response::error_raw(
                                        &StatusCode::BAD_REQUEST,
                                        &err.to_string(),
                                    )
                                }
                            };

                            let mut user_id = None;
                            if let Some(id) = user_data.get("_id") {
                                if let ColumnValue::Uuid(id) = id {
                                    if let Some(id) = id {
                                        user_id = Some(*id);
                                    }
                                }
                            }

                            if user_id.is_none() {
                                return Response::error_raw(
                                    &StatusCode::BAD_REQUEST,
                                    "User not found",
                                );
                            }

                            user_id
                        }
                        None => {
                            if let Some(token_data) = token_data {
                                Some(*token_data.id())
                            } else {
                                return Response::error_raw(
                                    &StatusCode::INTERNAL_SERVER_ERROR,
                                    "Cannot determine created_by",
                                );
                            }
                        }
                    },
                    BucketPermission::None => {
                        return Response::error_raw(
                            &StatusCode::BAD_REQUEST,
                            "User doesn't have permission to read this file",
                        )
                    }
                }
            } else {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    "User doesn't have permission to read this file",
                );
            };

            if let Some(created_by) = &created_by {
                if created_by != file_data.created_by() {
                    return Response::error_raw(
                        &StatusCode::FORBIDDEN,
                        "User doesn't have permission to read this file",
                    );
                }
            }

            if file_data.bucket_id() != bucket_data.id() {
                return Response::error_raw(&StatusCode::BAD_REQUEST, "Bucket id does not match");
            }
        } else {
            return Response::error_raw(
                &StatusCode::BAD_REQUEST,
                "User doesn't have permission to read this file",
            );
        }
    }

    if query.data().is_some() {
        Response::data(
            &StatusCode::OK,
            &None,
            FileResJson::new(
                file_data.id(),
                file_data.created_by(),
                file_data.created_at(),
                file_data.updated_at(),
                file_data.bucket_id(),
                file_data.file_name(),
                &file_data.content_type().to_string(),
                file_data.size(),
                file_data.public(),
            ),
        )
    } else {
        let file = match NamedFile::open_async(&format!(
            "{}/{}",
            bucket_data.path(),
            file_data.id()
        ))
        .await
        {
            Ok(file) => file,
            Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
        };

        let mut res = file.into_response(&req);
        res.headers_mut().insert(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!(
                "attachment; filename=\"{}\"",
                file_data.file_name()
            ))
            .unwrap(),
        );
        res
    }
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

    let (admin_id, token_data, user_claim) = match token_claim.id() {
        ClaimId::Admin(id) => match AdminDao::db_select(ctx.dao().db(), id).await {
            Ok(data) => (*data.id(), None, None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::UNAUTHORIZED,
                    &format!("Failed to get admin data: {err}"),
                )
            }
        },
        ClaimId::Token(token_id, user_claim) => {
            match TokenDao::db_select(ctx.dao().db(), token_id).await {
                Ok(data) => (*data.admin_id(), Some(data), *user_claim),
                Err(err) => {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        &format!("Failed to get token data: {err}"),
                    )
                }
            }
        }
    };

    let rule_update_one = if let Some(token_data) = &token_data {
        if let Some(rule) = token_data
            .is_allow_update_file(ctx.dao().db(), path.bucket_id())
            .await
        {
            Some(rule)
        } else {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to update this file",
            );
        }
    } else {
        None
    };

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

    let created_by = if matches!(token_claim.id(), ClaimId::Admin(_)) {
        None
    } else if let Some(rule) = rule_update_one {
        match rule {
            BucketPermission::All => None,
            BucketPermission::SelfMade => match user_claim {
                Some(user_claim) => {
                    let collection_data =
                        match CollectionDao::db_select(ctx.dao().db(), user_claim.collection_id())
                            .await
                        {
                            Ok(data) => data,
                            Err(err) => {
                                return Response::error_raw(
                                    &StatusCode::BAD_REQUEST,
                                    &err.to_string(),
                                )
                            }
                        };
                    let user_data = match RecordDao::db_select(
                        ctx.dao().db(),
                        user_claim.id(),
                        &None,
                        &HashSet::from_iter(["_id"]),
                        &collection_data,
                        &token_data.is_none(),
                    )
                    .await
                    {
                        Ok(data) => data,
                        Err(err) => {
                            return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string())
                        }
                    };

                    let mut user_id = None;
                    if let Some(id) = user_data.get("_id") {
                        if let ColumnValue::Uuid(id) = id {
                            if let Some(id) = id {
                                user_id = Some(*id);
                            }
                        }
                    }

                    if user_id.is_none() {
                        return Response::error_raw(&StatusCode::BAD_REQUEST, "User not found");
                    }

                    user_id
                }
                None => {
                    if let Some(token_data) = token_data {
                        Some(*token_data.id())
                    } else {
                        return Response::error_raw(
                            &StatusCode::INTERNAL_SERVER_ERROR,
                            "Cannot determine created_by",
                        );
                    }
                }
            },
            BucketPermission::None => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    "User doesn't have permission to update this file",
                )
            }
        }
    } else {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            "User doesn't have permission to update this file",
        );
    };

    let mut file_data = match FileDao::db_select(ctx.dao().db(), &bucket_data, path.file_id()).await
    {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if let Some(created_by) = &created_by {
        if created_by != file_data.created_by() {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "User doesn't have permission to update this file",
            );
        }
    }

    if file_data.bucket_id() != bucket_data.id() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Bucket id does not match");
    }

    if let Some(created_by) = data.created_by() {
        file_data.set_created_by(created_by);
    }

    if let Some(file_name) = data.file_name() {
        file_data.set_file_name(file_name);
    }

    if let Some(public) = data.public() {
        file_data.set_public(public);
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
            file_data.created_by(),
            file_data.created_at(),
            file_data.updated_at(),
            file_data.bucket_id(),
            file_data.file_name(),
            &file_data.content_type().to_string(),
            file_data.size(),
            file_data.public(),
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

    let (admin_id, token_data, user_claim) = match token_claim.id() {
        ClaimId::Admin(id) => match AdminDao::db_select(ctx.dao().db(), id).await {
            Ok(data) => (*data.id(), None, None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::UNAUTHORIZED,
                    &format!("Failed to get admin data: {err}"),
                )
            }
        },
        ClaimId::Token(token_id, user_claim) => {
            match TokenDao::db_select(ctx.dao().db(), token_id).await {
                Ok(data) => (*data.admin_id(), Some(data), *user_claim),
                Err(err) => {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        &format!("Failed to get token data: {err}"),
                    )
                }
            }
        }
    };

    let rule_delete_one = if let Some(token_data) = &token_data {
        if let Some(rule) = token_data
            .is_allow_delete_file(ctx.dao().db(), path.bucket_id())
            .await
        {
            Some(rule)
        } else {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to delete this file",
            );
        }
    } else {
        None
    };

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

    let created_by = if matches!(token_claim.id(), ClaimId::Admin(_)) {
        None
    } else if let Some(rule) = rule_delete_one {
        match rule {
            BucketPermission::All => None,
            BucketPermission::SelfMade => match user_claim {
                Some(user_claim) => {
                    let collection_data =
                        match CollectionDao::db_select(ctx.dao().db(), user_claim.collection_id())
                            .await
                        {
                            Ok(data) => data,
                            Err(err) => {
                                return Response::error_raw(
                                    &StatusCode::BAD_REQUEST,
                                    &err.to_string(),
                                )
                            }
                        };
                    let user_data = match RecordDao::db_select(
                        ctx.dao().db(),
                        user_claim.id(),
                        &None,
                        &HashSet::from_iter(["_id"]),
                        &collection_data,
                        &token_data.is_none(),
                    )
                    .await
                    {
                        Ok(data) => data,
                        Err(err) => {
                            return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string())
                        }
                    };

                    let mut user_id = None;
                    if let Some(id) = user_data.get("_id") {
                        if let ColumnValue::Uuid(id) = id {
                            if let Some(id) = id {
                                user_id = Some(*id);
                            }
                        }
                    }

                    if user_id.is_none() {
                        return Response::error_raw(&StatusCode::BAD_REQUEST, "User not found");
                    }

                    user_id
                }
                None => {
                    if let Some(token_data) = token_data {
                        Some(*token_data.id())
                    } else {
                        return Response::error_raw(
                            &StatusCode::INTERNAL_SERVER_ERROR,
                            "Cannot determine created_by",
                        );
                    }
                }
            },
            BucketPermission::None => {
                return Response::error_raw(
                    &StatusCode::BAD_REQUEST,
                    "User doesn't have permission to delete this file",
                )
            }
        }
    } else {
        return Response::error_raw(
            &StatusCode::BAD_REQUEST,
            "User doesn't have permission to delete this file",
        );
    };

    let file_data = match FileDao::db_select(ctx.dao().db(), &bucket_data, path.file_id()).await {
        Ok(data) => data,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    if let Some(created_by) = &created_by {
        if created_by != file_data.created_by() {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "User doesn't have permission to delete this file",
            );
        }
    }

    if file_data.bucket_id() != bucket_data.id() {
        return Response::error_raw(&StatusCode::BAD_REQUEST, "Bucket id does not match");
    }

    if let Err(err) = FileDao::delete(ctx.dao().db(), bucket_data.path(), path.file_id()).await {
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
    query: web::Query<FindManyFileReqQuery>,
) -> HttpResponse {
    let token = auth.token();

    let token_claim = match ctx.token().jwt().decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
    };

    let (admin_id, token_data, user_claim) = match token_claim.id() {
        ClaimId::Admin(id) => match AdminDao::db_select(ctx.dao().db(), id).await {
            Ok(data) => (*data.id(), None, None),
            Err(err) => {
                return Response::error_raw(
                    &StatusCode::UNAUTHORIZED,
                    &format!("Failed to get admin data: {err}"),
                )
            }
        },
        ClaimId::Token(token_id, user_claim) => {
            match TokenDao::db_select(ctx.dao().db(), token_id).await {
                Ok(data) => (*data.admin_id(), Some(data), *user_claim),
                Err(err) => {
                    return Response::error_raw(
                        &StatusCode::BAD_REQUEST,
                        &format!("Failed to get token data: {err}"),
                    )
                }
            }
        }
    };

    let rule_find_many = if let Some(token_data) = &token_data {
        if let Some(rule) = token_data
            .is_allow_find_many_files(ctx.dao().db(), path.bucket_id())
            .await
        {
            Some(rule)
        } else {
            return Response::error_raw(
                &StatusCode::FORBIDDEN,
                "This token doesn't have permission to read these files",
            );
        }
    } else {
        None
    };

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

    let created_by = if matches!(token_claim.id(), ClaimId::Admin(_)) {
        None
    } else if let Some(rule) = rule_find_many {
        match rule {
            BucketPermission::All => None,
            BucketPermission::SelfMade => match user_claim {
                Some(user_claim) => {
                    let collection_data =
                        match CollectionDao::db_select(ctx.dao().db(), user_claim.collection_id())
                            .await
                        {
                            Ok(data) => data,
                            Err(err) => {
                                return Response::error_raw(
                                    &StatusCode::BAD_REQUEST,
                                    &err.to_string(),
                                )
                            }
                        };
                    let user_data = match RecordDao::db_select(
                        ctx.dao().db(),
                        user_claim.id(),
                        &None,
                        &HashSet::from_iter(["_id"]),
                        &collection_data,
                        &token_data.is_none(),
                    )
                    .await
                    {
                        Ok(data) => data,
                        Err(err) => {
                            return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string())
                        }
                    };

                    let mut user_id = None;
                    if let Some(id) = user_data.get("_id") {
                        if let ColumnValue::Uuid(id) = id {
                            if let Some(id) = id {
                                user_id = Some(*id);
                            }
                        }
                    }

                    if user_id.is_none() {
                        return Response::error_raw(&StatusCode::BAD_REQUEST, "User not found");
                    }

                    user_id
                }
                None => {
                    if let Some(token_data) = token_data {
                        Some(*token_data.id())
                    } else {
                        return Response::error_raw(
                            &StatusCode::INTERNAL_SERVER_ERROR,
                            "Cannot determine created_by",
                        );
                    }
                }
            },
            BucketPermission::None => {
                return Response::error_raw(
                    &StatusCode::FORBIDDEN,
                    "User doesn't have permission to read these files",
                )
            }
        }
    } else {
        return Response::error_raw(
            &StatusCode::FORBIDDEN,
            "User doesn't have permission to read these files",
        );
    };

    let (files_data, total) = match created_by {
        Some(created_by) => {
            match FileDao::db_select_many_by_created_by_and_bucket_id(
                ctx.dao().db(),
                &created_by,
                &bucket_data,
                query.before_id(),
                query.limit(),
            )
            .await
            {
                Ok(data) => data,
                Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
            }
        }
        None => {
            match FileDao::db_select_many_by_bucket_id(
                ctx.dao().db(),
                &bucket_data,
                query.before_id(),
                query.limit(),
            )
            .await
            {
                Ok(data) => data,
                Err(err) => return Response::error_raw(&StatusCode::BAD_REQUEST, &err.to_string()),
            }
        }
    };

    let total = match usize::try_from(total) {
        Ok(data) => data,
        Err(err) => {
            return Response::error_raw(&StatusCode::INTERNAL_SERVER_ERROR, &err.to_string())
        }
    };

    Response::data(
        &StatusCode::OK,
        &Some(PaginationRes::new(&files_data.len(), &total)),
        &files_data
            .iter()
            .map(|data| {
                FileResJson::new(
                    data.id(),
                    data.created_by(),
                    data.created_at(),
                    data.updated_at(),
                    data.bucket_id(),
                    data.file_name(),
                    &data.content_type().to_string(),
                    data.size(),
                    data.public(),
                )
            })
            .collect::<Vec<_>>(),
    )
}
