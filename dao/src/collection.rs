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
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    collection_rule::CollectionRuleDao, record::RecordDao, util::conversion, value::ColumnKind, Db,
};

#[derive(Deserialize, Serialize)]
pub struct CollectionDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    project_id: Uuid,
    name: String,
    schema_fields: HashMap<String, SchemaFieldProps>,
    opt_auth_column_id: bool,
    opt_ttl: Option<i64>,
    #[serde(skip)]
    _preserve: Option<Preserve>,
}

impl CollectionDao {
    pub fn new(
        project_id: &Uuid,
        name: &str,
        schema_fields: &HashMap<String, SchemaFieldProps>,
        opt_auth_column_id: &bool,
        opt_ttl: &Option<i64>,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::now_v7(),
            created_at: now,
            updated_at: now,
            project_id: *project_id,
            name: name.to_owned(),
            schema_fields: schema_fields.clone(),
            opt_auth_column_id: *opt_auth_column_id,
            opt_ttl: *opt_ttl,
            _preserve: None,
        }
    }

    pub fn from_bytes<'a>(bytes: &'a [u8]) -> Result<Self, rmp_serde::decode::Error>
    where
        Self: Deserialize<'a>,
    {
        rmp_serde::from_slice(bytes)
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(self)
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

    pub fn opt_auth_column_id(&self) -> &bool {
        &self.opt_auth_column_id
    }

    pub fn opt_ttl(&self) -> &Option<i64> {
        &self.opt_ttl
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }

    pub fn update_schema_fields(&mut self, schema_fields: &HashMap<String, SchemaFieldProps>) {
        if self._preserve.is_none() {
            self._preserve = Some(Preserve {
                schema_fields: Some(self.schema_fields.clone()),
            });
        } else {
            self._preserve.as_mut().unwrap().schema_fields = Some(self.schema_fields.clone());
        }
        self.schema_fields = schema_fields.clone();
    }

    pub fn set_opt_auth_column_id(&mut self, opt_auth_column_id: &bool) {
        self.opt_auth_column_id = *opt_auth_column_id;
    }

    pub fn set_opt_ttl(&mut self, opt_ttl: &Option<i64>) {
        if let Some(opt_ttl) = opt_ttl {
            if *opt_ttl <= 0 {
                self.opt_ttl = None;
                return;
            }
        }
        self.opt_ttl = *opt_ttl;
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        let mut create_indexes_fut = Vec::with_capacity(self.schema_fields.len());
        let mut create_unique_indexes_fut = Vec::with_capacity(self.schema_fields.len());

        match db {
            Db::ScyllaDb(_) => {
                for (field, props) in &self.schema_fields {
                    if props.unique {
                        return Err(Error::msg(format!(
                            "Field '{field}' requires unique index but ScyllaDB doesn't support unique indexes"
                        )));
                    }
                }
            }
            Db::MysqlDb(_) => {
                for (field, props) in &self.schema_fields {
                    if props.unique || props.indexed {
                        match &props.kind {
                        ColumnKind::Binary
                        | ColumnKind::Varint
                        | ColumnKind::Decimal
                        | ColumnKind::String
                        | ColumnKind::Json => {
                            return Err(Error::msg(format!(
                                "Field '{}' has type '{}' that doesn't support indexing in the data type implementation of Hyperbase for MySQL",
                                field,
                                props.kind.to_str()
                            )))
                        }
                        _ => (),
                    };
                        if props.indexed {
                            create_indexes_fut
                                .push(RecordDao::db_create_index(db, &self.id, field));
                        }
                        if props.unique {
                            create_unique_indexes_fut
                                .push(RecordDao::db_create_unique_index(db, &self.id, field));
                        }
                    }
                }
            }
            _ => (),
        }

        RecordDao::db_create_table(db, self).await?;

        tokio::try_join!(
            future::try_join_all(create_indexes_fut),
            future::try_join_all(create_unique_indexes_fut)
        )?;

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
                    collections_data.push(Self::from_scylladb_model(&collection?)?);
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

    pub async fn db_select_many_from_updated_at_and_after_id_with_limit_asc(
        db: &Db,
        updated_at: &DateTime<Utc>,
        id: &Uuid,
        limit: &i32,
    ) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(_) => Err(Error::msg("Unimplemented")),
            Db::PostgresqlDb(db) => {
                let collections = db
                    .select_many_collections_from_updated_at_and_after_id_with_limit_asc(
                        updated_at, id, limit,
                    )
                    .await?;
                let mut collections_data = Vec::with_capacity(collections.len());
                for collection in &collections {
                    collections_data.push(Self::from_postgresdb_model(collection)?);
                }
                Ok(collections_data)
            }
            Db::MysqlDb(db) => {
                let collections = db
                    .select_many_collections_from_updated_at_and_after_id_with_limit_asc(
                        updated_at, id, limit,
                    )
                    .await?;
                let mut collections_data = Vec::with_capacity(collections.len());
                for collection in &collections {
                    collections_data.push(Self::from_mysqldb_model(collection)?);
                }
                Ok(collections_data)
            }
            Db::SqliteDb(db) => {
                let collections = db
                    .select_many_collections_from_updated_at_and_after_id_with_limit_asc(
                        updated_at, id, limit,
                    )
                    .await?;
                let mut collections_data = Vec::with_capacity(collections.len());
                for collection in &collections {
                    collections_data.push(Self::from_sqlitedb_model(collection)?);
                }
                Ok(collections_data)
            }
        }
    }

    async fn db_update_prepare(&mut self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(_) => {
                for (field, props) in &self.schema_fields {
                    if props.unique {
                        return Err(Error::msg(format!(
                            "Field '{field}' requires unique index but ScyllaDB doesn't support unique indexes"
                        )));
                    }
                }
            }
            Db::MysqlDb(_) => {
                for (field, props) in &self.schema_fields {
                    if props.indexed {
                        match &props.kind {
                        ColumnKind::Binary
                        | ColumnKind::Varint
                        | ColumnKind::Decimal
                        | ColumnKind::String
                        | ColumnKind::Json => {
                            return Err(Error::msg(format!(
                                "Field '{}' has type '{}' that doesn't support indexing in the data type implementation of Hyperbase for MySQL",
                                field,
                                props.kind.to_str()
                            )))
                        }
                        _ => (),
                    };
                    }
                }
            }
            _ => (),
        }

        let mut already_indexed = HashSet::new();
        let mut already_unique_indexed = HashSet::new();

        if let Some(preserve) = &self._preserve {
            if let Some(preserved_schema_fields) = &preserve.schema_fields {
                let mut drop_indexes_fut = Vec::with_capacity(preserved_schema_fields.len());
                let mut drop_unique_indexes_fut = Vec::with_capacity(preserved_schema_fields.len());

                let mut columns_change_type = HashMap::with_capacity(self.schema_fields.len());
                let mut columns_drop = HashSet::with_capacity(preserved_schema_fields.len());

                for (field_name, field_props) in preserved_schema_fields {
                    match self.schema_fields.get(field_name) {
                        Some(props) => {
                            if field_props.indexed {
                                if props.indexed {
                                    already_indexed.insert(field_name.as_str());
                                } else {
                                    drop_indexes_fut
                                        .push(RecordDao::db_drop_index(db, &self.id, field_name));
                                }
                            }
                            if field_props.unique {
                                if props.unique {
                                    already_unique_indexed.insert(field_name.as_str());
                                } else {
                                    drop_unique_indexes_fut.push(RecordDao::db_drop_unique_index(
                                        db, &self.id, field_name,
                                    ));
                                }
                            }
                            if field_props.kind() != props.kind() {
                                columns_change_type.insert(field_name.to_owned(), *props);
                            }
                        }
                        None => {
                            if field_props.indexed {
                                drop_indexes_fut
                                    .push(RecordDao::db_drop_index(db, &self.id, field_name));
                            }
                            if field_props.unique {
                                drop_unique_indexes_fut
                                    .push(RecordDao::db_drop_unique_index(db, &self.id, field_name))
                            }
                            columns_drop.insert(field_name.clone());
                        }
                    };
                }
                if !drop_indexes_fut.is_empty() {
                    future::try_join_all(drop_indexes_fut).await?;
                }
                if !drop_unique_indexes_fut.is_empty() {
                    future::try_join_all(drop_unique_indexes_fut).await?;
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
        }

        let mut create_indexes_fut = Vec::with_capacity(self.schema_fields.len());
        let mut create_unique_indexes_fut = Vec::with_capacity(self.schema_fields.len());
        for (field, props) in &self.schema_fields {
            if props.indexed && !already_indexed.contains(field.as_str()) {
                create_indexes_fut.push(RecordDao::db_create_index(db, &self.id, field));
            }
            if props.unique && !already_unique_indexed.contains(field.as_str()) {
                create_unique_indexes_fut
                    .push(RecordDao::db_create_unique_index(db, &self.id, field))
            }
        }

        tokio::try_join!(
            future::try_join_all(create_indexes_fut),
            future::try_join_all(create_unique_indexes_fut)
        )?;

        Ok(())
    }

    pub async fn db_update(&mut self, db: &Db) -> Result<()> {
        self.db_update_prepare(db).await?;

        self.updated_at = Utc::now();

        match db {
            Db::ScyllaDb(db) => db.update_collection(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.update_collection(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.update_collection(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.update_collection(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_update_raw(&mut self, db: &Db) -> Result<()> {
        self.db_update_prepare(db).await?;

        match db {
            Db::ScyllaDb(db) => db.update_collection(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.update_collection(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.update_collection(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.update_collection(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_delete(db: &Db, id: &Uuid) -> Result<()> {
        RecordDao::db_drop_table(db, id).await?;

        CollectionRuleDao::db_delete_many_by_collection_id(db, id).await?;

        match db {
            Db::ScyllaDb(db) => db.delete_collection(id).await,
            Db::PostgresqlDb(db) => db.delete_collection(id).await,
            Db::MysqlDb(db) => db.delete_collection(id).await,
            Db::SqliteDb(db) => db.delete_collection(id).await,
        }
    }

    fn from_scylladb_model(model: &CollectionScyllaModel) -> Result<Self> {
        let mut schema_fields = HashMap::new();
        if let Some(model_schema_fields) = model.schema_fields() {
            schema_fields = HashMap::with_capacity(model_schema_fields.len());
            for (key, value) in model_schema_fields {
                let value = match SchemaFieldProps::from_scylladb_model(value) {
                    Ok(value) => value,
                    Err(err) => return Err(err),
                };
                schema_fields.insert(key.to_owned(), value);
            }
        }
        Ok(Self {
            id: *model.id(),
            created_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.created_at())?,
            updated_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.updated_at())?,
            project_id: *model.project_id(),
            name: model.name().to_owned(),
            schema_fields,
            opt_auth_column_id: *model.opt_auth_column_id(),
            opt_ttl: *model.opt_ttl(),
            _preserve: None,
        })
    }

    fn to_scylladb_model(&self) -> CollectionScyllaModel {
        CollectionScyllaModel::new(
            &self.id,
            &conversion::datetime_utc_to_scylla_cql_timestamp(&self.created_at),
            &conversion::datetime_utc_to_scylla_cql_timestamp(&self.updated_at),
            &self.project_id,
            &self.name,
            &Some(
                self.schema_fields
                    .iter()
                    .map(|(key, value)| (key.to_owned(), value.to_scylladb_model()))
                    .collect(),
            ),
            &self.opt_auth_column_id,
            &self.opt_ttl,
        )
    }

    fn from_postgresdb_model(model: &CollectionPostgresModel) -> Result<Self> {
        let mut schema_fields = HashMap::with_capacity(model.schema_fields().len());
        for (key, value) in &model.schema_fields().0 {
            let value = match SchemaFieldProps::from_postgresdb_model(value) {
                Ok(value) => value,
                Err(err) => return Err(err),
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
            opt_auth_column_id: *model.opt_auth_column_id(),
            opt_ttl: *model.opt_ttl(),
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
            &self.opt_auth_column_id,
            &self.opt_ttl,
        )
    }

    fn from_mysqldb_model(model: &CollectionMysqlModel) -> Result<Self> {
        let mut schema_fields = HashMap::with_capacity(model.schema_fields().len());
        for (key, value) in &model.schema_fields().0 {
            let value = match SchemaFieldProps::from_mysqldb_model(value) {
                Ok(value) => value,
                Err(err) => return Err(err),
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
            opt_auth_column_id: *model.opt_auth_column_id(),
            opt_ttl: *model.opt_ttl(),
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
            &self.opt_auth_column_id,
            &self.opt_ttl,
        )
    }

    fn from_sqlitedb_model(model: &CollectionSqliteModel) -> Result<Self> {
        let mut schema_fields = HashMap::with_capacity(model.schema_fields().len());
        for (key, value) in &model.schema_fields().0 {
            let value = match SchemaFieldProps::from_sqlitedb_model(value) {
                Ok(value) => value,
                Err(err) => return Err(err),
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
            opt_auth_column_id: *model.opt_auth_column_id(),
            opt_ttl: *model.opt_ttl(),
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
            &self.opt_auth_column_id,
            &self.opt_ttl,
        )
    }
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct SchemaFieldProps {
    kind: ColumnKind,
    required: bool,
    unique: bool,
    indexed: bool,
    auth_column: bool,
    hidden: bool,
}

impl SchemaFieldProps {
    pub fn new(
        kind: &ColumnKind,
        required: &bool,
        unique: &bool,
        indexed: &bool,
        auth_column: &bool,
        hidden: &bool,
    ) -> Self {
        Self {
            kind: *kind,
            required: *required,
            unique: *unique,
            indexed: *indexed,
            auth_column: *auth_column,
            hidden: *hidden,
        }
    }

    pub fn kind(&self) -> &ColumnKind {
        &self.kind
    }

    pub fn required(&self) -> &bool {
        &self.required
    }

    pub fn unique(&self) -> &bool {
        &self.unique
    }

    pub fn indexed(&self) -> &bool {
        &self.indexed
    }

    pub fn auth_column(&self) -> &bool {
        &self.auth_column
    }

    pub fn hidden(&self) -> &bool {
        &self.hidden
    }

    fn from_scylladb_model(model: &SchemaFieldPropsScyllaModel) -> Result<Self> {
        let kind = match ColumnKind::from_str(model.kind()) {
            Ok(kind) => kind,
            Err(err) => return Err(err),
        };
        Ok(Self {
            kind,
            required: *model.required(),
            unique: *model.unique(),
            indexed: *model.indexed(),
            auth_column: *model.auth_column(),
            hidden: *model.hidden(),
        })
    }

    pub fn to_scylladb_model(&self) -> SchemaFieldPropsScyllaModel {
        SchemaFieldPropsScyllaModel::new(
            self.kind.to_str(),
            &self.kind.to_scylladb_model(),
            &self.required,
            &self.unique,
            &self.indexed,
            &self.auth_column,
            &self.hidden,
        )
    }

    fn from_postgresdb_model(model: &SchemaFieldPropsPostgresModel) -> Result<Self> {
        let kind = match ColumnKind::from_str(model.kind()) {
            Ok(kind) => kind,
            Err(err) => return Err(err),
        };
        Ok(Self {
            kind,
            required: *model.required(),
            unique: *model.unique(),
            indexed: *model.indexed(),
            auth_column: *model.auth_column(),
            hidden: *model.hidden(),
        })
    }

    pub fn to_postgresdb_model(&self) -> SchemaFieldPropsPostgresModel {
        SchemaFieldPropsPostgresModel::new(
            self.kind.to_str(),
            &self.kind.to_postgresdb_model(),
            &self.required,
            &self.unique,
            &self.indexed,
            &self.auth_column,
            &self.hidden,
        )
    }

    fn from_mysqldb_model(model: &SchemaFieldPropsMysqlModel) -> Result<Self> {
        let kind = match ColumnKind::from_str(model.kind()) {
            Ok(kind) => kind,
            Err(err) => return Err(err),
        };
        Ok(Self {
            kind,
            required: *model.required(),
            unique: *model.unique(),
            indexed: *model.indexed(),
            auth_column: *model.auth_column(),
            hidden: *model.hidden(),
        })
    }

    pub fn to_mysqldb_model(&self) -> SchemaFieldPropsMysqlModel {
        SchemaFieldPropsMysqlModel::new(
            self.kind.to_str(),
            &self.kind.to_mysqldb_model(),
            &self.required,
            &self.unique,
            &self.indexed,
            &self.auth_column,
            &self.hidden,
        )
    }

    fn from_sqlitedb_model(model: &SchemaFieldPropsSqliteModel) -> Result<Self> {
        let kind = match ColumnKind::from_str(model.kind()) {
            Ok(kind) => kind,
            Err(err) => return Err(err),
        };
        Ok(Self {
            kind,
            required: *model.required(),
            unique: *model.unique(),
            indexed: *model.indexed(),
            auth_column: *model.auth_column(),
            hidden: *model.hidden(),
        })
    }

    pub fn to_sqlitedb_model(&self) -> SchemaFieldPropsSqliteModel {
        SchemaFieldPropsSqliteModel::new(
            self.kind.to_str(),
            &self.kind.to_sqlitedb_model(),
            &self.required,
            &self.unique,
            &self.indexed,
            &self.auth_column,
            &self.hidden,
        )
    }
}

struct Preserve {
    schema_fields: Option<HashMap<String, SchemaFieldProps>>,
}
