use ahash::HashMap;
use anyhow::{Error, Result};
use hb_dao::{
    collection::CollectionDao,
    record::{RecordColumnValue, RecordFilter, RecordFilters},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InsertOneRecordReqPath {
    project_id: Uuid,
    collection_id: Uuid,
}

impl InsertOneRecordReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }
}

pub type InsertOneRecordReqJson = HashMap<String, Value>;

#[derive(Deserialize)]
pub struct FindOneRecordReqPath {
    project_id: Uuid,
    collection_id: Uuid,
    record_id: Uuid,
}

impl FindOneRecordReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn record_id(&self) -> &Uuid {
        &self.record_id
    }
}

#[derive(Deserialize)]
pub struct UpdateOneRecordReqPath {
    project_id: Uuid,
    collection_id: Uuid,
    record_id: Uuid,
}

impl UpdateOneRecordReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn record_id(&self) -> &Uuid {
        &self.record_id
    }
}

pub type UpdateOneRecordReqJson = HashMap<String, Value>;

#[derive(Deserialize)]
pub struct DeleteOneRecordReqPath {
    project_id: Uuid,
    collection_id: Uuid,
    record_id: Uuid,
}

impl DeleteOneRecordReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }

    pub fn record_id(&self) -> &Uuid {
        &self.record_id
    }
}

#[derive(Deserialize)]
pub struct FindManyRecordReqPath {
    project_id: Uuid,
    collection_id: Uuid,
}

impl FindManyRecordReqPath {
    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn collection_id(&self) -> &Uuid {
        &self.collection_id
    }
}

#[derive(Deserialize)]
pub struct FindManyRecordReqJson {
    filter: Option<FindManyRecordFiltersReqJson>,
    order: Option<Vec<FindManyRecordOrderReqJson>>,
    limit: Option<i32>,
}

impl FindManyRecordReqJson {
    pub fn filter(&self) -> &Option<FindManyRecordFiltersReqJson> {
        &self.filter
    }

    pub fn order(&self) -> &Option<Vec<FindManyRecordOrderReqJson>> {
        &self.order
    }

    pub fn limit(&self) -> &Option<i32> {
        &self.limit
    }
}

#[derive(Deserialize)]
pub struct FindManyRecordFiltersReqJson(Vec<FindManyRecordFilterReqJson>);

impl FindManyRecordFiltersReqJson {
    pub fn to_dao(&self, collection_data: &CollectionDao) -> Result<RecordFilters> {
        let mut filters = Vec::<RecordFilter>::with_capacity(self.0.len());
        for f in &self.0 {
            if (f.field.is_some() || f.value.is_some()) && f.child.is_some() {
                return Err(Error::msg("Wrong filter format. If 'child' field exists, then 'name' and 'value' fields must not exist"));
            } else if f.child.is_none() && (f.field.is_none() || f.value.is_none()) {
                return Err(Error::msg("Wrong filter format. If 'child' field does not exist, then 'name' and 'value' fields must exist"));
            }
            let schema_field_kind = match &f.field {
                Some(field) => match collection_data.schema_fields().get(field) {
                    Some(field) => Some(field.kind()),
                    None => {
                        return Err(Error::msg(format!(
                            "Field '{field}' is not exist in the collection",
                        )));
                    }
                },
                None => None,
            };

            let value = if schema_field_kind.is_some() && f.value.is_some() {
                match RecordColumnValue::from_serde_json(
                    schema_field_kind.unwrap(),
                    f.value.as_ref().unwrap(),
                ) {
                    Ok(value) => Some(value),
                    Err(err) => {
                        return Err(err);
                    }
                }
            } else {
                None
            };
            filters.push(RecordFilter::new(
                &f.field,
                &f.op,
                &value,
                &if let Some(child) = &f.child {
                    Some(child.to_dao(collection_data)?)
                } else {
                    None
                },
            ));
        }
        Ok(RecordFilters::new(&filters))
    }
}

#[derive(Deserialize)]
pub struct FindManyRecordFilterReqJson {
    field: Option<String>,
    op: String,
    value: Option<serde_json::Value>,
    child: Option<FindManyRecordFiltersReqJson>,
}

#[derive(Deserialize)]
pub struct FindManyRecordOrderReqJson {
    field: String,
    kind: String,
}

impl FindManyRecordOrderReqJson {
    pub fn field(&self) -> &str {
        &self.field
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }
}

#[derive(Serialize)]
pub struct RecordResJson {
    #[serde(flatten)]
    data: HashMap<String, Value>,
}

impl RecordResJson {
    pub fn new(data: &HashMap<String, Value>) -> Self {
        Self { data: data.clone() }
    }
}

#[derive(Serialize)]
pub struct DeleteRecordResJson {
    id: Uuid,
}

impl DeleteRecordResJson {
    pub fn new(id: &Uuid) -> Self {
        Self { id: *id }
    }
}
