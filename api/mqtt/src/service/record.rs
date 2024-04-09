use std::sync::Arc;

use ahash::{HashMap, HashMapExt, HashSet};
use anyhow::{Error, Result};
use hb_api_websocket::server::{Message as WebSocketMessage, MessageKind as WebSocketMessageKind};
use hb_dao::{
    collection::CollectionDao,
    log::{LogDao, LogKind},
    project::ProjectDao,
    record::RecordDao,
    token::TokenDao,
    value::ColumnValue,
};
use uuid::Uuid;

use crate::{context::ApiMqttCtx, model::payload::Payload};

pub async fn record_service(ctx: &Arc<ApiMqttCtx>, payload: &Payload) {
    match insert_one(ctx.clone(), payload).await {
        Ok(_) => {
            let msg = format!(
                "ApiMqttClient: Successfully insert one payload to collection_id {}",
                payload.collection_id()
            );
            hb_log::info(None, &msg);
            if let Ok(token_data) = TokenDao::db_select(ctx.dao().db(), payload.token_id()).await {
                let log_data = LogDao::new(
                    token_data.admin_id(),
                    payload.project_id(),
                    &LogKind::Info,
                    &msg,
                );
                let _ = log_data.db_insert(ctx.dao().db()).await;
            };
        }
        Err(err) => {
            let err_msg = format!("ApiMqttClient: {err}");
            hb_log::error(None, &err_msg);
            if let Ok(token_data) = TokenDao::db_select(ctx.dao().db(), payload.token_id()).await {
                let log_data = LogDao::new(
                    token_data.admin_id(),
                    payload.project_id(),
                    &LogKind::Error,
                    &err_msg,
                );
                let _ = log_data.db_insert(ctx.dao().db()).await;
            };
        }
    };
}

async fn insert_one(ctx: Arc<ApiMqttCtx>, payload: &Payload) -> Result<()> {
    let token_data = match TokenDao::db_select(ctx.dao().db(), payload.token_id()).await {
        Ok(data) => data,
        Err(err) => return Err(Error::msg(format!("Failed to get token data: {err}"))),
    };

    if !token_data
        .is_allow_insert_record(ctx.dao().db(), payload.collection_id())
        .await
    {
        return Err(Error::msg(format!(
            "Token id '{}' doesn't have permission to write data to collection id {}",
            payload.token_id(),
            payload.collection_id()
        )));
    }

    let (project_data, collection_data) = tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), payload.project_id()),
        CollectionDao::db_select(ctx.dao().db(), payload.collection_id())
    )?;

    if token_data.admin_id() != project_data.admin_id() {
        return Err(Error::msg(format!(
            "This project id '{}' doesn't belong to you (token id '{}'",
            payload.project_id(),
            payload.token_id()
        )));
    }

    if project_data.id() != collection_data.project_id() {
        return Err(Error::msg(format!(
            "Project id ({}) does not match",
            project_data.id()
        )));
    }

    for field_name in payload.data().keys() {
        if !collection_data.schema_fields().contains_key(field_name) {
            return Err(Error::msg(format!(
                "Field '{field_name}' is not exist in the collection ({})",
                payload.collection_id()
            )));
        }
    }

    let created_by = if let Some(user_claim) = payload.user() {
        let collection_data =
            match CollectionDao::db_select(ctx.dao().db(), user_claim.collection_id()).await {
                Ok(data) => data,
                Err(err) => return Err(err),
            };
        if !collection_data.opt_auth_column_id() {
            return Err(Error::msg(format!(
                "Authentication using field '_id' on collection id '{}' is disabled",
                collection_data.id()
            )));
        }
        let user_data = match RecordDao::db_select(
            ctx.dao().db(),
            user_claim.id(),
            &None,
            &HashSet::from_iter(["_id"]),
            &collection_data,
        )
        .await
        {
            Ok(data) => data,
            Err(err) => return Err(err),
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
            return Err(Error::msg("User doesn't found"));
        }
    } else if *token_data.allow_anonymous() {
        *token_data.admin_id()
    } else {
        return Err(Error::msg(format!(
            "User with token id '{}' doesn't have permission to write data to this collection",
            payload.token_id()
        )));
    };

    let mut record_data = RecordDao::new(
        &created_by,
        collection_data.id(),
        &Some(payload.data().len()),
    );
    for (field_name, field_props) in collection_data.schema_fields() {
        if let Some(value) = payload.data().get(field_name) {
            if !value.is_null() {
                record_data.upsert(
                    field_name,
                    &match ColumnValue::from_serde_json(field_props.kind(), value) {
                        Ok(value) => value,
                        Err(err) => {
                            return Err(Error::msg(
                                format!("Error in field '{field_name}': {err}",),
                            ))
                        }
                    },
                );
                continue;
            }
        }
        if *field_props.required() {
            return Err(Error::msg(format!("Value for '{field_name}' is required")));
        } else {
            record_data.upsert(field_name, &ColumnValue::none(field_props.kind()));
        }
    }

    record_data.db_insert(ctx.dao().db()).await?;

    tokio::spawn((|| async move {
        let mut record = HashMap::with_capacity(record_data.len());
        for (key, value) in record_data.data() {
            let value = match value.to_serde_json() {
                Ok(value) => value,
                Err(err) => {
                    hb_log::error(
                        None,
                        &format!("ApiMqttClient: Error when serializing record: {err}"),
                    );
                    return;
                }
            };
            record.insert(key.to_owned(), value);
        }

        let record_id = Uuid::parse_str(record["_id"].as_str().unwrap()).unwrap();
        let created_by = Uuid::parse_str(record["_created_by"].as_str().unwrap()).unwrap();

        let record = match serde_json::to_value(&record) {
            Ok(value) => value,
            Err(err) => {
                hb_log::error(
                    None,
                    &format!("ApiMqttClient: Error when serializing record {record_id}: {err}"),
                );
                return;
            }
        };

        if let Err(err) = ctx.websocket().broadcaster().broadcast(
            *collection_data.id(),
            WebSocketMessage::new(
                *collection_data.id(),
                record_id,
                created_by,
                WebSocketMessageKind::InsertOne,
                record,
            ),
        ) {
            hb_log::error(
                    None,
                    &format!(
                        "ApiMqttClient: Error when broadcasting insert_one record {record_id} to websocket: {err}"
                    ),
                );
            return;
        }
    })());

    Ok(())
}
