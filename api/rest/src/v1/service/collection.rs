use std::str::FromStr;

use actix_web::{http::StatusCode, web, HttpResponse};
use hb_dao::{
    collection::{CollectionDao, SchemaFieldKind, SchemaFieldModel},
    Db,
};
use hb_token_jwt::kind::JwtTokenKind;

use crate::{
    context::ApiRestContext as Context,
    v1::model::{
        collection::{
            CollectionResJson, DeleteOneCollectionReqPath, FindOneCollectionReqPath,
            InsertOneCollectionReqJson, InsertOneCollectionReqPath, SchemaFieldModelJson,
            UpdateOneCollectionReqPath,
        },
        Response, TokenReqHeader,
    },
};

pub fn collection_api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/project/{project_id}/collection")
            .route("", web::post().to(insert_one))
            .route("/{collection_id}", web::get().to(find_one))
            .route("/{collection_id}", web::patch().to(update_one))
            .route("/{collection_id}", web::patch().to(delete_one)),
    );

    cfg.service(
        web::scope("/project/{project_id}/collections").route("", web::get().to(find_many)),
    );
}

async fn insert_one(
    ctx: web::Data<Context>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<InsertOneCollectionReqPath>,
    data: web::Json<InsertOneCollectionReqJson>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token.jwt.decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str()),
    };

    if token_claim.kind() != &JwtTokenKind::Admin {
        return Response::error(StatusCode::BAD_REQUEST, "Must be logged in as admin");
    }

    let db = Db::ScyllaDb(&ctx.db.scylladb);

    let mut schema_fields = Vec::new();
    for field in data.schema_fields().iter() {
        schema_fields.push(SchemaFieldModel::new(
            field.name(),
            match &SchemaFieldKind::from_str(field.kind()) {
                Ok(kind) => kind,
                Err(err) => {
                    return Response::error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        err.to_string().as_str(),
                    )
                }
            },
            field.required(),
        ))
    }

    let collection_data = CollectionDao::new(
        path.project_id(),
        data.name(),
        &schema_fields,
        data.indexes(),
    );

    if let Err(err) = collection_data.insert(&db).await {
        return Response::error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string().as_str());
    }

    Response::data(
        StatusCode::OK,
        None,
        CollectionResJson::new(
            collection_data.id(),
            collection_data.created_at(),
            collection_data.updated_at(),
            collection_data.project_id(),
            collection_data.name(),
            &collection_data
                .schema_fields()
                .iter()
                .map(|field| {
                    SchemaFieldModelJson::new(
                        field.name(),
                        field.kind().to_string().as_str(),
                        field.required(),
                    )
                })
                .collect::<Vec<_>>(),
            collection_data.indexes(),
        ),
    )
}

async fn find_one(
    ctx: web::Data<Context>,
    token: web::Header<TokenReqHeader>,
    path: web::Path<FindOneCollectionReqPath>,
) -> HttpResponse {
    let token = match token.get() {
        Some(token) => token,
        None => return Response::error(StatusCode::BAD_REQUEST, "Invalid token"),
    };

    let token_claim = match ctx.token.jwt.decode(token) {
        Ok(token) => token,
        Err(err) => return Response::error(StatusCode::BAD_REQUEST, err.to_string().as_str()),
    };

    if token_claim.kind() != &JwtTokenKind::Admin {
        return Response::error(StatusCode::BAD_REQUEST, "Must be logged in as admin");
    }

    let db = Db::ScyllaDb(&ctx.db.scylladb);

    let collection_data = match CollectionDao::select(&db, path.collection_id())
    HttpResponse::Ok().body(format!("collection find_one: {}", path.collection_id()))
}

async fn update_one(
    path: web::Path<UpdateOneCollectionReqPath>,
    data: web::Json<InsertOneCollectionReqJson>,
) -> HttpResponse {
    HttpResponse::Ok().body(format!(
        "collection update_one: {}, {}",
        path.collection_id(),
        data.name()
    ))
}

async fn delete_one(path: web::Path<DeleteOneCollectionReqPath>) -> HttpResponse {
    HttpResponse::Ok().body(format!("collection delete_one: {}", path.collection_id()))
}

async fn find_many() -> HttpResponse {
    HttpResponse::Ok().body("collection find_many")
}
