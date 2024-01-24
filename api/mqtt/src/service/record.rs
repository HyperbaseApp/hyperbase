use anyhow::{Error, Result};
use hb_dao::{
    collection::CollectionDao,
    project::ProjectDao,
    record::RecordDao,
    token::TokenDao,
    value::{ColumnKind, ColumnValue},
};

use crate::{
    context::ApiMqttCtx,
    model::payload::{Method, Payload},
};

pub async fn record_service(ctx: &ApiMqttCtx, payload: &Payload) {
    match payload.method() {
        Method::InsertOne => match insert_one(ctx, payload).await {
            Ok(_) => hb_log::info(None, "ApiMqttClient: Successfully insert one payload"),
            Err(err) => hb_log::error(None, err),
        },
    }
}

async fn insert_one(ctx: &ApiMqttCtx, payload: &Payload) -> Result<()> {
    let token_data = match TokenDao::db_select_by_token(ctx.dao().db(), payload.token()).await {
        Ok(data) => data,
        Err(err) => return Err(Error::msg(format!("Failed to get token data: {err}"))),
    };

    if !token_data.is_allow_insert_record(payload.collection_id()) {
        return Err(Error::msg(format!(
            "Token ({}) doesn't have permission to write data to this collection",
            payload.token()
        )));
    }

    let (project_data, collection_data) = tokio::try_join!(
        ProjectDao::db_select(ctx.dao().db(), payload.project_id()),
        CollectionDao::db_select(ctx.dao().db(), payload.collection_id())
    )?;

    if token_data.admin_id() != project_data.admin_id() {
        return Err(Error::msg(format!(
            "This project (id: {}) does not belong to you (token: {})",
            payload.project_id(),
            payload.token()
        )));
    }

    if project_data.id() != collection_data.project_id() {
        return Err(Error::msg(format!(
            "Project id ({}) does not match",
            project_data.id()
        )));
    }

    if let Some(data) = payload.data() {
        for field_name in data.keys() {
            if !collection_data.schema_fields().contains_key(field_name) {
                return Err(Error::msg(format!(
                    "Field '{field_name}' is not exist in the collection ({})",
                    payload.collection_id()
                )));
            }
        }

        let mut record_data = RecordDao::new(collection_data.id(), &Some(data.len()));
        for (field_name, field_props) in collection_data.schema_fields() {
            if let Some(value) = data.get(field_name) {
                if !value.is_null() {
                    if let Some(value) = value.as_str() {
                        if value == "$request.auth.id" {
                            if *field_props.kind() != ColumnKind::Uuid {
                                return Err(Error::msg(
                                    "Field for storing '$request.auth.id' must be of type 'uuid'",
                                ));
                            }
                            record_data
                                .upsert(field_name, &ColumnValue::Uuid(Some(*token_data.id())));
                            continue;
                        }
                    }
                    record_data.upsert(
                        field_name,
                        &match ColumnValue::from_serde_json(field_props.kind(), value) {
                            Ok(value) => value,
                            Err(err) => {
                                return Err(Error::msg(format!(
                                    "Error in field '{}': {}",
                                    field_name, err
                                )))
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
    } else {
        return Err(Error::msg("'data' field in payload is required"));
    }

    Ok(())
}
