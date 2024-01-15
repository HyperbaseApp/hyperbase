use std::{str::FromStr, sync::Arc};

use ahash::{HashMap, HashMapExt};
use hb_dao::{
    collection::CollectionDao,
    project::ProjectDao,
    record::RecordDao,
    token::TokenDao,
    value::{ColumnKind, ColumnValue},
};
use ntex::{fn_service, service::fn_factory_with_config, util::Ready};
use ntex_mqtt::v5;
use uuid::Uuid;

use crate::{
    context::ApiMqttCtx,
    error_handler::ServerError,
    model::record::{InsertOneRecordReqJson, UpdateOneRecordReqJson},
    session::Session,
};

pub fn v5_pub_record_api(
    ctx: Arc<ApiMqttCtx>,
    router: v5::Router<Session, ServerError>,
) -> v5::Router<Session, ServerError> {
    let ctx_insert = ctx.clone();
    let ctx_update = ctx.clone();
    let ctx_delete = ctx.clone();

    router
        .resource(
            ["hb/{project_id}/{collection_id}/insert"],
            fn_factory_with_config(move |session: v5::Session<Session>| {
                let ctx = ctx_insert.clone();
                Ready::Ok::<_, ServerError>(fn_service(move |publish| {
                    insert_one(ctx.clone(), session.clone(), publish)
                }))
            }),
        )
        .resource(
            ["hb/{project_id}/{collection_id}/{record_id}/update"],
            fn_factory_with_config(move |session: v5::Session<Session>| {
                let ctx = ctx_update.clone();
                Ready::Ok::<_, ServerError>(fn_service(move |publish| {
                    update_one(ctx.clone(), session.clone(), publish)
                }))
            }),
        )
        .resource(
            ["hb/{project_id}/{collection_id}/{record_id}/delete"],
            fn_factory_with_config(move |session: v5::Session<Session>| {
                let ctx = ctx_delete.clone();
                Ready::Ok::<_, ServerError>(fn_service(move |publish| {
                    delete_one(ctx.clone(), session.clone(), publish)
                }))
            }),
        )
}

async fn insert_one(
    ctx: Arc<ApiMqttCtx>,
    session: v5::Session<Session>,
    publish: v5::Publish,
) -> Result<v5::PublishAck, ServerError> {
    let project_id = Uuid::from_str(
        publish
            .topic()
            .get("project_id")
            .ok_or_else(|| ServerError)?,
    )
    .map_err(|_| ServerError)?;
    let collection_id = Uuid::from_str(
        publish
            .topic()
            .get("collection_id")
            .ok_or_else(|| ServerError)?,
    )
    .map_err(|_| ServerError)?;
    let data = serde_json::from_str::<InsertOneRecordReqJson>(
        std::str::from_utf8(&publish.packet().payload).map_err(|_| ServerError)?,
    )
    .map_err(|_| ServerError)?;

    let token_data = TokenDao::db_select(ctx.dao().db(), session.token_id())
        .await
        .map_err(|_| ServerError)?;

    if !token_data.is_allow_insert_record(&collection_id) {
        return Err(ServerError);
    }

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), &project_id),
        CollectionDao::db_select(ctx.dao().db(), &collection_id)
    ) {
        Ok(data) => data,
        Err(_) => return Err(ServerError),
    };

    if token_data.admin_id() != project_data.admin_id() {
        return Err(ServerError);
    }

    if project_data.id() != collection_data.project_id() {
        return Err(ServerError);
    }

    for field_name in data.keys() {
        if !collection_data.schema_fields().contains_key(field_name) {
            return Err(ServerError);
        }
    }

    let mut record_data = RecordDao::new(collection_data.id(), &Some(data.len()));
    for (field_name, field_props) in collection_data.schema_fields() {
        if let Some(value) = data.get(field_name) {
            if !value.is_null() {
                if let Some(value) = value.as_str() {
                    if value == "$request.auth.id" {
                        if *field_props.kind() != ColumnKind::Uuid {
                            return Err(ServerError);
                        }
                        record_data.upsert(field_name, &ColumnValue::Uuid(Some(*token_data.id())));
                        continue;
                    }
                }
                record_data.upsert(
                    field_name,
                    &match ColumnValue::from_serde_json(field_props.kind(), value) {
                        Ok(value) => value,
                        Err(_) => return Err(ServerError),
                    },
                );
                continue;
            }
        }
        if *field_props.required() {
            return Err(ServerError);
        } else {
            record_data.upsert(field_name, &ColumnValue::none(field_props.kind()));
        }
    }

    if let Err(_) = record_data.db_insert(ctx.dao().db()).await {
        return Err(ServerError);
    }

    let mut record = HashMap::with_capacity(record_data.len());
    for (key, value) in record_data.data() {
        let value = match value.to_serde_json() {
            Ok(value) => value,
            Err(_) => return Err(ServerError),
        };
        record.insert(key.to_owned(), value);
    }

    println!(
        ">>>v5_insert_one client_id={} token_id={} project_id={} collection_id={}",
        session.state().client_id(),
        session.state().token_id(),
        project_id,
        collection_id
    );

    Ok(publish.ack())
}

async fn update_one(
    ctx: Arc<ApiMqttCtx>,
    session: v5::Session<Session>,
    publish: v5::Publish,
) -> Result<v5::PublishAck, ServerError> {
    let project_id = Uuid::from_str(
        publish
            .topic()
            .get("project_id")
            .ok_or_else(|| ServerError)?,
    )
    .map_err(|_| ServerError)?;
    let collection_id = Uuid::from_str(
        publish
            .topic()
            .get("collection_id")
            .ok_or_else(|| ServerError)?,
    )
    .map_err(|_| ServerError)?;
    let record_id = Uuid::from_str(
        publish
            .topic()
            .get("record_id")
            .ok_or_else(|| ServerError)?,
    )
    .map_err(|_| ServerError)?;
    let data = serde_json::from_str::<UpdateOneRecordReqJson>(
        std::str::from_utf8(&publish.packet().payload).map_err(|_| ServerError)?,
    )
    .map_err(|_| ServerError)?;

    let token_data = TokenDao::db_select(ctx.dao().db(), session.token_id())
        .await
        .map_err(|_| ServerError)?;

    if !token_data.is_allow_update_record(&collection_id) {
        return Err(ServerError);
    }

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), &project_id),
        CollectionDao::db_select(ctx.dao().db(), &collection_id),
    ) {
        Ok(data) => data,
        Err(_) => return Err(ServerError),
    };

    if token_data.admin_id() != project_data.admin_id() {
        return Err(ServerError);
    }

    if project_data.id() != collection_data.project_id() {
        return Err(ServerError);
    }

    for field_name in data.keys() {
        if !collection_data.schema_fields().contains_key(field_name) {
            return Err(ServerError);
        }
    }

    let mut record_data =
        match RecordDao::db_select(ctx.dao().db(), &collection_data, &record_id).await {
            Ok(data) => data,
            Err(_) => return Err(ServerError),
        };
    for (field_name, field_props) in collection_data.schema_fields() {
        if let Some(value) = data.get(field_name) {
            if value.is_null() {
                if *field_props.required() {
                    return Err(ServerError);
                }
            }
            if let Some(value) = value.as_str() {
                if value == "$request.auth.id" {
                    if *field_props.kind() != ColumnKind::Uuid {
                        return Err(ServerError);
                    }
                    record_data.upsert(field_name, &ColumnValue::Uuid(Some(*token_data.id())));
                    continue;
                }
            }
            record_data.upsert(
                field_name,
                &match ColumnValue::from_serde_json(field_props.kind(), value) {
                    Ok(value) => value,
                    Err(_) => return Err(ServerError),
                },
            );
        }
    }

    if let Err(_) = record_data.db_update(ctx.dao().db()).await {
        return Err(ServerError);
    }

    let mut record = HashMap::with_capacity(record_data.len());
    for (key, value) in record_data.data() {
        let value = match value.to_serde_json() {
            Ok(value) => value,
            Err(_) => return Err(ServerError),
        };
        record.insert(key.to_owned(), value);
    }

    println!(
        ">>>v5_update_one client_id={} token_id={} project_id={} collection_id={}",
        session.state().client_id(),
        session.state().token_id(),
        project_id,
        collection_id
    );

    Ok(publish.ack())
}

async fn delete_one(
    ctx: Arc<ApiMqttCtx>,
    session: v5::Session<Session>,
    publish: v5::Publish,
) -> Result<v5::PublishAck, ServerError> {
    let project_id = Uuid::from_str(
        publish
            .topic()
            .get("project_id")
            .ok_or_else(|| ServerError)?,
    )
    .map_err(|_| ServerError)?;
    let collection_id = Uuid::from_str(
        publish
            .topic()
            .get("collection_id")
            .ok_or_else(|| ServerError)?,
    )
    .map_err(|_| ServerError)?;
    let record_id = Uuid::from_str(
        publish
            .topic()
            .get("record_id")
            .ok_or_else(|| ServerError)?,
    )
    .map_err(|_| ServerError)?;

    let token_data = TokenDao::db_select(ctx.dao().db(), session.token_id())
        .await
        .map_err(|_| ServerError)?;

    if !token_data.is_allow_insert_record(&collection_id) {
        return Err(ServerError);
    }

    let (project_data, collection_data) = match tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), &project_id),
        CollectionDao::db_select(ctx.dao().db(), &collection_id),
    ) {
        Ok(data) => data,
        Err(_) => return Err(ServerError),
    };

    if token_data.admin_id() != project_data.admin_id() {
        return Err(ServerError);
    }

    if project_data.id() != collection_data.project_id() {
        return Err(ServerError);
    }

    if let Err(_) = RecordDao::db_delete(ctx.dao().db(), collection_data.id(), &record_id).await {
        return Err(ServerError);
    }

    println!(
        ">>>v5_delete_one client_id={} token_id={} project_id={} collection_id={}",
        session.state().client_id(),
        session.state().token_id(),
        project_id,
        collection_id
    );

    Ok(publish.ack())
}
