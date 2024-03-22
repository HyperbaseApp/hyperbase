use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use futures::future;
use hb_db_mysql::model::collection::{
    CollectionModel as CollectionMysqlModel, SchemaFieldPropsModel as SchemaFieldPropsMysqlModel,
};
use hb_db_postgresql::model::collection::{
    CollectionModel as CollectionPostgresModel,
    SchemaFieldPropsModel as SchemaFieldPropsPostgresModel,
};
use hb_db_scylladb::model::collection::{
    CollectionModel as CollectionScyllaModel, SchemaFieldPropsModel as SchemaFieldPropsScyllaModel,
};
use hb_db_sqlite::model::collection::{
    CollectionModel as CollectionSqliteModel, SchemaFieldPropsModel as SchemaFieldPropsSqliteModel,
};
use scylla::frame::value::CqlTimestamp as ScyllaCqlTimestamp;
use serde::Serialize;
use uuid::Uuid;

use crate::{record::RecordDao, util::conversion, value::ColumnKind, Db};

pub struct CollectionDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    project_id: Uuid,
    name: String,
    schema_fields: HashMap<String, SchemaFieldProps>,
    indexes: HashSet<String>,
    auth_columns: HashSet<String>,
    _preserve: Option<Preserve>,
}

impl CollectionDao {
    pub fn new(
        project_id: &Uuid,
        name: &str,
        schema_fields: &HashMap<String, SchemaFieldProps>,
        indexes: &HashSet<String>,
        auth_columns: &HashSet<String>,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::now_v7(),
            created_at: now,
            updated_at: now,
            project_id: *project_id,
            name: name.to_owned(),
            schema_fields: schema_fields.clone(),
            indexes: indexes.clone(),
            auth_columns: auth_columns.clone(),
            _preserve: None,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn schema_fields(&self) -> &HashMap<String, SchemaFieldProps> {
        &self.schema_fields
    }

    pub fn indexes(&self) -> &HashSet<String> {
        &self.indexes
    }

    pub fn auth_columns(&self) -> &HashSet<String> {
        &self.auth_columns
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }

    pub fn update_schema_fields(&mut self, schema_fields: &HashMap<String, SchemaFieldProps>) {
        if self._preserve.is_none() {
            self._preserve = Some(Preserve {
                schema_fields: Some(self.schema_fields.clone()),
                indexes: None,
            });
        } else {
            self._preserve.as_mut().unwrap().schema_fields = Some(self.schema_fields.clone());
        }
        self.schema_fields = schema_fields.clone();
    }

    pub fn update_indexes(&mut self, indexes: &HashSet<String>) {
        if self._preserve.is_none() {
            self._preserve = Some(Preserve {
                schema_fields: None,
                indexes: Some(self.indexes.clone()),
            });
        } else {
            self._preserve.as_mut().unwrap().indexes = Some(self.indexes.clone());
        }
        self.indexes = indexes.to_owned();
    }

    pub fn set_auth_columns(&mut self, auth_columns: &HashSet<String>) {
        self.auth_columns = auth_columns.clone();
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        if let Db::MysqlDb(_) = db {
            for index in &self.indexes {
                if let Some(field) = self.schema_fields.get(index) {
                    match &field.kind {
                        ColumnKind::Binary
                        | ColumnKind::Varint
                        | ColumnKind::Decimal
                        | ColumnKind::String
                        | ColumnKind::Json => {
                            return Err(Error::msg(format!(
                                "Field '{}' has type '{}' that doesn't support indexing in the data type implementation of Hyperbase for MySQL",
                                index,
                                field.kind.to_str()
                            )))
                        }
                        _ => (),
                    };
                }
            }
        }

        RecordDao::db_create_table(db, self).await?;

        let mut create_indexes_fut = Vec::with_capacity(self.indexes.len());
        for index in &self.indexes {
            create_indexes_fut.push(RecordDao::db_create_index(db, &self.id, index));
        }
        future::try_join_all(create_indexes_fut).await?;

        match db {
            Db::ScyllaDb(db) => db.insert_collection(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.insert_collection(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.insert_collection(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.insert_collection(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_select(db: &Db, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Ok(Self::from_scylladb_model(&db.select_collection(id).await?)?),
            Db::PostgresqlDb(db) => Ok(Self::from_postgresdb_model(
                &db.select_collection(id).await?,
            )?),
            Db::MysqlDb(db) => Ok(Self::from_mysqldb_model(&db.select_collection(id).await?)?),
            Db::SqliteDb(db) => Ok(Self::from_sqlitedb_model(&db.select_collection(id).await?)?),
        }
    }

    pub async fn db_select_many_by_project_id(db: &Db, project_id: &Uuid) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(db) => {
                let mut collections_data = Vec::new();
                let collections = db.select_many_collections_by_project_id(project_id).await?;
                for collection in collections {
                    collections_data.push(Self::from_scylladb_model(&collection?)?)
                }
                Ok(collections_data)
            }
            Db::PostgresqlDb(db) => {
                let collections = db.select_many_collections_by_project_id(project_id).await?;
                let mut collections_data = Vec::with_capacity(collections.len());
                for collection in &collections {
                    collections_data.push(Self::from_postgresdb_model(collection)?);
                }
                Ok(collections_data)
            }
            Db::MysqlDb(db) => {
                let collections = db.select_many_collections_by_project_id(project_id).await?;
                let mut collections_data = Vec::with_capacity(collections.len());
                for collection in &collections {
                    collections_data.push(Self::from_mysqldb_model(collection)?);
                }
                Ok(collections_data)
            }
            Db::SqliteDb(db) => {
                let collections = db.select_many_collections_by_project_id(project_id).await?;
                let mut collections_data = Vec::with_capacity(collections.len());
                for collection in &collections {
                    collections_data.push(Self::from_sqlitedb_model(collection)?);
                }
                Ok(collections_data)
            }
        }
    }

    pub async fn db_update(&mut self, db: &Db) -> Result<()> {
        if let Db::MysqlDb(_) = db {
            for index in &self.indexes {
                if let Some(field) = self.schema_fields.get(index) {
                    match &field.kind {
                        ColumnKind::Binary
                        | ColumnKind::Varint
                        | ColumnKind::Decimal
                        | ColumnKind::String
                        | ColumnKind::Json => {
                            return Err(Error::msg(format!(
                                "Field '{}' has type '{}' that doesn't support indexing in the data type implementation of Hyperbase for MySQL",
                                index,
                                field.kind.to_str()
                            )))
                        }
                        _ => (),
                    };
                }
            }
        }

        let is_preserve_schema_fields_exist = self
            ._preserve
            .as_ref()
            .is_some_and(|preserve| preserve.schema_fields.as_ref().is_some());
        let is_preserve_indexes_exist = self
            ._preserve
            .as_ref()
            .is_some_and(|preserve| preserve.indexes.as_ref().is_some());

        if is_preserve_indexes_exist {
            let mut drop_indexes_fut = Vec::new();
            for index in self._preserve.as_ref().unwrap().indexes.as_ref().unwrap() {
                if !self.indexes.contains(index) {
                    drop_indexes_fut.push(RecordDao::db_drop_index(db, &self.id, index));
                }
            }
            future::try_join_all(drop_indexes_fut).await?;
        }

        if is_preserve_schema_fields_exist {
            let mut columns_change_type = HashMap::new();
            let mut columns_drop = HashSet::new();
            for (field_name, field_props) in self
                ._preserve
                .as_ref()
                .unwrap()
                .schema_fields
                .as_ref()
                .unwrap()
            {
                match self.schema_fields.get(field_name) {
                    Some(field) => {
                        if field.kind() != field_props.kind() {
                            columns_change_type.insert(field_name.to_owned(), *field);
                        }
                    }
                    None => {
                        columns_drop.insert(field_name.clone());
                    }
                };
            }
            if !columns_change_type.is_empty() {
                RecordDao::db_change_columns_type(db, &self.id, &columns_change_type).await?;
            }
            if !columns_drop.is_empty() {
                RecordDao::db_drop_columns(db, &self.id, &columns_drop).await?;
            }

            let mut columns_add = HashMap::new();
            for (field_name, field_props) in &self.schema_fields {
                if !self
                    ._preserve
                    .as_ref()
                    .unwrap()
                    .schema_fields
                    .as_ref()
                    .unwrap()
                    .contains_key(field_name)
                {
                    columns_add.insert(field_name.to_owned(), *field_props);
                }
            }
            if !columns_add.is_empty() {
                RecordDao::db_add_columns(db, &self.id, &columns_add).await?;
            }
        }

        if is_preserve_indexes_exist {
            let mut create_indexes_fut = Vec::new();
            for index in &self.indexes {
                if !self
                    ._preserve
                    .as_ref()
                    .unwrap()
                    .indexes
                    .as_ref()
                    .unwrap()
                    .contains(index)
                {
                    create_indexes_fut.push(RecordDao::db_create_index(db, &self.id, index));
                }
            }
            future::try_join_all(create_indexes_fut).await?;
        }

        self.updated_at = Utc::now();

        match db {
            Db::ScyllaDb(db) => db.update_collection(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.update_collection(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.update_collection(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.update_collection(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_delete(db: &Db, id: &Uuid) -> Result<()> {
        RecordDao::db_drop_table(db, id).await?;

        match db {
            Db::ScyllaDb(db) => db.delete_collection(id).await,
            Db::PostgresqlDb(db) => db.delete_collection(id).await,
            Db::MysqlDb(db) => db.delete_collection(id).await,
            Db::SqliteDb(db) => db.delete_collection(id).await,
        }
    }

    fn from_scylladb_model(model: &CollectionScyllaModel) -> Result<Self> {
        let mut schema_fields = HashMap::with_capacity(model.schema_fields().len());
        for (key, value) in model.schema_fields() {
            let value = match SchemaFieldProps::from_scylladb_model(value) {
                Ok(value) => value,
                Err(err) => return Err(err.into()),
            };
            schema_fields.insert(key.to_owned(), value);
        }
        Ok(Self {
            id: *model.id(),
            created_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.created_at())?,
            updated_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.updated_at())?,
            project_id: *model.project_id(),
            name: model.name().to_owned(),
            schema_fields,
            indexes: match model.indexes() {
                Some(indexes) => indexes.to_owned(),
                None => HashSet::new(),
            },
            auth_columns: match model.auth_columns() {
                Some(auth_columns) => auth_columns.to_owned(),
                None => HashSet::new(),
            },
            _preserve: None,
        })
    }

    fn to_scylladb_model(&self) -> CollectionScyllaModel {
        CollectionScyllaModel::new(
            &self.id,
            &ScyllaCqlTimestamp(self.created_at.timestamp_millis()),
            &ScyllaCqlTimestamp(self.updated_at.timestamp_millis()),
            &self.project_id,
            &self.name,
            &self
                .schema_fields
                .iter()
                .map(|(key, value)| (key.to_owned(), value.to_scylladb_model()))
                .collect(),
            &if self.indexes.len() > 0 {
                Some(self.indexes.clone())
            } else {
                None
            },
            &if self.auth_columns.len() > 0 {
                Some(self.auth_columns.clone())
            } else {
                None
            },
        )
    }

    fn from_postgresdb_model(model: &CollectionPostgresModel) -> Result<Self> {
        let mut schema_fields = HashMap::with_capacity(model.schema_fields().len());
        for (key, value) in &model.schema_fields().0 {
            let value = match SchemaFieldProps::from_postgresdb_model(value) {
                Ok(value) => value,
                Err(err) => return Err(err.into()),
            };
            schema_fields.insert(key.to_owned(), value);
        }
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            project_id: *model.project_id(),
            name: model.name().to_owned(),
            schema_fields,
            indexes: model.indexes().0.to_owned(),
            auth_columns: model.auth_columns().0.to_owned(),
            _preserve: None,
        })
    }

    fn to_postgresdb_model(&self) -> CollectionPostgresModel {
        CollectionPostgresModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.project_id,
            &self.name,
            &sqlx::types::Json(
                self.schema_fields
                    .iter()
                    .map(|(key, value)| (key.to_owned(), value.to_postgresdb_model()))
                    .collect(),
            ),
            &sqlx::types::Json(self.indexes.to_owned()),
            &sqlx::types::Json(self.auth_columns.to_owned()),
        )
    }

    fn from_mysqldb_model(model: &CollectionMysqlModel) -> Result<Self> {
        let mut schema_fields = HashMap::with_capacity(model.schema_fields().len());
        for (key, value) in &model.schema_fields().0 {
            let value = match SchemaFieldProps::from_mysqldb_model(value) {
                Ok(value) => value,
                Err(err) => return Err(err.into()),
            };
            schema_fields.insert(key.to_owned(), value);
        }
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            project_id: *model.project_id(),
            name: model.name().to_owned(),
            schema_fields,
            indexes: model.indexes().0.to_owned(),
            auth_columns: model.auth_columns().0.to_owned(),
            _preserve: None,
        })
    }

    fn to_mysqldb_model(&self) -> CollectionMysqlModel {
        CollectionMysqlModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.project_id,
            &self.name,
            &sqlx::types::Json(
                self.schema_fields
                    .iter()
                    .map(|(key, value)| (key.to_owned(), value.to_mysqldb_model()))
                    .collect(),
            ),
            &sqlx::types::Json(self.indexes.to_owned()),
            &sqlx::types::Json(self.auth_columns.to_owned()),
        )
    }

    fn from_sqlitedb_model(model: &CollectionSqliteModel) -> Result<Self> {
        let mut schema_fields = HashMap::with_capacity(model.schema_fields().len());
        for (key, value) in &model.schema_fields().0 {
            let value = match SchemaFieldProps::from_sqlitedb_model(value) {
                Ok(value) => value,
                Err(err) => return Err(err.into()),
            };
            schema_fields.insert(key.to_owned(), value);
        }
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            project_id: *model.project_id(),
            name: model.name().to_owned(),
            schema_fields,
            indexes: model.indexes().0.to_owned(),
            auth_columns: model.auth_columns().0.to_owned(),
            _preserve: None,
        })
    }

    fn to_sqlitedb_model(&self) -> CollectionSqliteModel {
        CollectionSqliteModel::new(
            &self.id,
            &self.created_at,
            &self.updated_at,
            &self.project_id,
            &self.name,
            &sqlx::types::Json(
                self.schema_fields
                    .iter()
                    .map(|(key, value)| (key.to_owned(), value.to_sqlitedb_model()))
                    .collect(),
            ),
            &sqlx::types::Json(self.indexes.to_owned()),
            &sqlx::types::Json(self.auth_columns.to_owned()),
        )
    }
}

#[derive(Serialize, Clone, Copy)]
pub struct SchemaFieldProps {
    kind: ColumnKind,
    required: bool,
}

impl SchemaFieldProps {
    pub fn new(kind: &ColumnKind, required: &bool) -> Self {
        Self {
            kind: *kind,
            required: *required,
        }
    }

    pub fn kind(&self) -> &ColumnKind {
        &self.kind
    }

    pub fn required(&self) -> &bool {
        &self.required
    }

    fn from_scylladb_model(model: &SchemaFieldPropsScyllaModel) -> Result<Self> {
        let kind = match ColumnKind::from_str(model.kind()) {
            Ok(kind) => kind,
            Err(err) => return Err(err.into()),
        };
        Ok(Self {
            kind,
            required: *model.required(),
        })
    }

    pub fn to_scylladb_model(&self) -> SchemaFieldPropsScyllaModel {
        SchemaFieldPropsScyllaModel::new(
            self.kind.to_str(),
            &self.kind.to_scylladb_model(),
            &self.required,
        )
    }

    fn from_postgresdb_model(model: &SchemaFieldPropsPostgresModel) -> Result<Self> {
        let kind = match ColumnKind::from_str(model.kind()) {
            Ok(kind) => kind,
            Err(err) => return Err(err.into()),
        };
        Ok(Self {
            kind,
            required: *model.required(),
        })
    }

    pub fn to_postgresdb_model(&self) -> SchemaFieldPropsPostgresModel {
        SchemaFieldPropsPostgresModel::new(
            self.kind.to_str(),
            &self.kind.to_postgresdb_model(),
            &self.required,
        )
    }

    fn from_mysqldb_model(model: &SchemaFieldPropsMysqlModel) -> Result<Self> {
        let kind = match ColumnKind::from_str(model.kind()) {
            Ok(kind) => kind,
            Err(err) => return Err(err.into()),
        };
        Ok(Self {
            kind,
            required: *model.required(),
        })
    }

    pub fn to_mysqldb_model(&self) -> SchemaFieldPropsMysqlModel {
        SchemaFieldPropsMysqlModel::new(
            self.kind.to_str(),
            &self.kind.to_mysqldb_model(),
            &self.required,
        )
    }

    fn from_sqlitedb_model(model: &SchemaFieldPropsSqliteModel) -> Result<Self> {
        let kind = match ColumnKind::from_str(model.kind()) {
            Ok(kind) => kind,
            Err(err) => return Err(err.into()),
        };
        Ok(Self {
            kind,
            required: *model.required(),
        })
    }

    pub fn to_sqlitedb_model(&self) -> SchemaFieldPropsSqliteModel {
        SchemaFieldPropsSqliteModel::new(
            self.kind.to_str(),
            &self.kind.to_sqlitedb_model(),
            &self.required,
        )
    }
}

struct Preserve {
    schema_fields: Option<HashMap<String, SchemaFieldProps>>,
    indexes: Option<HashSet<String>>,
}
