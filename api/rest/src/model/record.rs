use ahash::{HashMap, HashSet};
use anyhow::{Error, Result};
use hb_dao::{
    collection::CollectionDao,
    record::{RecordFilter, RecordFilters},
    value::{ColumnKind, ColumnValue},
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
pub struct FindOneRecordReqQuery {
    fields: Option<HashSet<String>>,
}

impl FindOneRecordReqQuery {
    pub fn fields(&self) -> &Option<HashSet<String>> {
        &self.fields
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
    fields: Option<HashSet<String>>,
    filters: Option<FindManyRecordFiltersReqJson>,
    groups: Option<Vec<String>>,
    orders: Option<Vec<FindManyRecordOrderReqJson>>,
    limit: Option<i32>,
}

impl FindManyRecordReqJson {
    pub fn fields(&self) -> &Option<HashSet<String>> {
        &self.fields
    }

    pub fn filters(&self) -> &Option<FindManyRecordFiltersReqJson> {
        &self.filters
    }

    pub fn groups(&self) -> &Option<Vec<String>> {
        &self.groups
    }

    pub fn orders(&self) -> &Option<Vec<FindManyRecordOrderReqJson>> {
        &self.orders
    }

    pub fn limit(&self) -> &Option<i32> {
        &self.limit
    }
}

#[derive(Deserialize)]
pub struct FindManyRecordFiltersReqJson(Vec<FindManyRecordFilterReqJson>);

impl FindManyRecordFiltersReqJson {
    pub fn to_dao(&self, collection_data: &CollectionDao) -> Result<RecordFilters> {
        let mut filters = Vec::with_capacity(self.0.len());
        for f in &self.0 {
            if (f.field.is_some() || f.value.is_some()) && f.children.is_some() {
                return Err(Error::msg("Wrong filter format. If 'children' field exists, then 'field' and 'value' fields must not exist"));
            } else if f.children.is_none() && f.field.is_none() {
                return Err(Error::msg("Wrong filter format. If 'children' field does not exist, then 'field' field must exist"));
            }
            let schema_field_kind = match &f.field {
                Some(field) => match collection_data.schema_fields().get(field) {
                    Some(field) => Some(field.kind()),
                    None => match field.as_str() {
                        "_id" | "_created_by" => Some(&ColumnKind::Uuid),
                        "_updated_at" => Some(&ColumnKind::Timestamp),
                        _ => {
                            return Err(Error::msg(format!(
                                "Field '{field}' is not exist in the collection",
                            )))
                        }
                    },
                },
                None => None,
            };

            let value = if schema_field_kind.is_some() && f.value.is_some() {
                match ColumnValue::from_serde_json(
                    schema_field_kind.unwrap(),
                    f.value.as_ref().unwrap(),
                ) {
                    Ok(value) => Some(value),
                    Err(err) => {
                        return Err(Error::msg(format!(
                            "Error in field '{}': {}",
                            f.field.as_ref().unwrap(),
                            err
                        )));
                    }
                }
            } else {
                None
            };
            filters.push(RecordFilter::new(
                &f.field,
                &f.op,
                &value,
                &if let Some(children) = &f.children {
                    Some(children.to_dao(collection_data)?)
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
    value: Option<Value>,
    children: Option<FindManyRecordFiltersReqJson>,
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
