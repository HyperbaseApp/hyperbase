use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use futures::future;
use hb_db_mysql::{
    db::MysqlDb,
    model::{
        collection::{
            CollectionModel as CollectionMysqlModel,
            SchemaFieldPropsModel as SchemaFieldPropsMysqlModel,
        },
        system::SchemaFieldKind as SchemaFieldMysqlKind,
    },
    query::collection::{
        DELETE as MYSQL_DELETE, INSERT as MYSQL_INSERT, SELECT as MYSQL_SELECT,
        SELECT_MANY_BY_PROJECT_ID as MYSQL_SELECT_MANY_BY_PROJECT_ID, UPDATE as MYSQL_UPDATE,
    },
};
use hb_db_postgresql::{
    db::PostgresDb,
    model::{
        collection::{
            CollectionModel as CollectionPostgresModel,
            SchemaFieldPropsModel as SchemaFieldPropsPostgresModel,
        },
        system::SchemaFieldKind as SchemaFieldPostgresKind,
    },
    query::collection::{
        DELETE as POSTGRES_DELETE, INSERT as POSTGRES_INSERT, SELECT as POSTGRES_SELECT,
        SELECT_MANY_BY_PROJECT_ID as POSTGRES_SELECT_MANY_BY_PROJECT_ID, UPDATE as POSTGRES_UPDATE,
    },
};
use hb_db_scylladb::{
    db::ScyllaDb,
    model::{
        collection::{
            CollectionModel as CollectionScyllaModel,
            SchemaFieldPropsModel as SchemaFieldPropsScyllaModel,
        },
        system::SchemaFieldKind as SchemaFieldScyllaKind,
    },
    query::collection::{
        DELETE as SCYLLA_DELETE, INSERT as SCYLLA_INSERT, SELECT as SCYLLA_SELECT,
        SELECT_MANY_BY_PROJECT_ID as SCYLLA_SELECT_MANY_BY_PROJECT_ID, UPDATE as SCYLLA_UPDATE,
    },
};
use hb_db_sqlite::{
    db::SqliteDb,
    model::{
        collection::{
            CollectionModel as CollectionSqliteModel,
            SchemaFieldPropsModel as SchemaFieldPropsSqliteModel,
        },
        system::SchemaFieldKind as SchemaFieldSqliteKind,
    },
    query::collection::{
        DELETE as SQLITE_DELETE, INSERT as SQLITE_INSERT, SELECT as SQLITE_SELECT,
        SELECT_MANY_BY_PROJECT_ID as SQLITE_SELECT_MANY_BY_PROJECT_ID, UPDATE as SQLITE_UPDATE,
    },
};
use scylla::{
    frame::value::CqlTimestamp as ScyllaCqlTimestamp,
    transport::session::TypedRowIter as ScyllaTypedRowIter,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{record::RecordDao, util::conversion, Db};

pub struct CollectionDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    project_id: Uuid,
    name: String,
    schema_fields: HashMap<String, SchemaFieldPropsModel>,
    indexes: HashSet<String>,
    _preserve: Option<Preserve>,
}

impl CollectionDao {
    pub fn new(
        project_id: &Uuid,
        name: &str,
        schema_fields: &HashMap<String, SchemaFieldPropsModel>,
        indexes: &HashSet<String>,
    ) -> Result<Self> {
        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            project_id: *project_id,
            name: name.to_string(),
            schema_fields: schema_fields.clone(),
            indexes: indexes.clone(),
            _preserve: None,
        })
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

    pub fn schema_fields(&self) -> &HashMap<String, SchemaFieldPropsModel> {
        &self.schema_fields
    }

    pub fn indexes(&self) -> &HashSet<String> {
        &self.indexes
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }

    pub fn update_schema_fields(&mut self, schema_fields: &HashMap<String, SchemaFieldPropsModel>) {
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

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        RecordDao::db_create_table(db, self).await?;

        let mut create_indexes_fut = Vec::with_capacity(self.indexes.len());
        for index in &self.indexes {
            create_indexes_fut.push(RecordDao::db_create_index(db, &self.id, index));
        }
        future::try_join_all(create_indexes_fut).await?;

        match db {
            Db::ScyllaDb(db) => Self::scylladb_insert(self, db).await,
            Db::PostgresqlDb(db) => Self::postgresdb_insert(self, db).await,
            Db::MysqlDb(db) => Self::mysqldb_insert(self, db).await,
            Db::SqliteDb(db) => Self::sqlitedb_insert(self, db).await,
        }
    }

    pub async fn db_select(db: &Db, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Ok(Self::from_scylladb_model(
                &Self::scylladb_select(db, id).await?,
            )?),
            Db::PostgresqlDb(db) => Ok(Self::from_postgresdb_model(
                &Self::postgresdb_select(db, id).await?,
            )?),
            Db::MysqlDb(db) => Ok(Self::from_mysqldb_model(
                &Self::mysqldb_select(db, id).await?,
            )?),
            Db::SqliteDb(db) => Ok(Self::from_sqlitedb_model(
                &Self::sqlitedb_select(db, id).await?,
            )?),
        }
    }

    pub async fn db_select_many_by_project_id(db: &Db, project_id: &Uuid) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(db) => {
                let mut collections_data = Vec::new();
                let collections = Self::scylladb_select_many_by_project_id(db, project_id).await?;
                for collection in collections {
                    collections_data.push(Self::from_scylladb_model(&collection?)?)
                }
                Ok(collections_data)
            }
            Db::PostgresqlDb(db) => {
                let collections =
                    Self::postgresdb_select_many_by_project_id(db, project_id).await?;
                let mut collections_data = Vec::with_capacity(collections.len());
                for collection in &collections {
                    collections_data.push(Self::from_postgresdb_model(collection)?);
                }
                Ok(collections_data)
            }
            Db::MysqlDb(db) => {
                let collections = Self::mysqldb_select_many_by_project_id(db, project_id).await?;
                let mut collections_data = Vec::with_capacity(collections.len());
                for collection in &collections {
                    collections_data.push(Self::from_mysqldb_model(collection)?);
                }
                Ok(collections_data)
            }
            Db::SqliteDb(db) => {
                let collections = Self::sqlitedb_select_many_by_project_id(db, project_id).await?;
                let mut collections_data = Vec::with_capacity(collections.len());
                for collection in &collections {
                    collections_data.push(Self::from_sqlitedb_model(collection)?);
                }
                Ok(collections_data)
            }
        }
    }

    pub async fn db_update(&mut self, db: &Db) -> Result<()> {
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
            Db::ScyllaDb(db) => Self::scylladb_update(self, db).await,
            Db::PostgresqlDb(db) => Self::postgresdb_update(self, db).await,
            Db::MysqlDb(db) => Self::mysqldb_update(self, db).await,
            Db::SqliteDb(db) => Self::sqlitedb_update(self, db).await,
        }
    }

    pub async fn db_delete(db: &Db, id: &Uuid) -> Result<()> {
        RecordDao::db_drop_table(db, id).await?;

        match db {
            Db::ScyllaDb(db) => Self::scylladb_delete(db, id).await,
            Db::PostgresqlDb(db) => Self::postgresdb_delete(db, id).await,
            Db::MysqlDb(db) => Self::mysqldb_delete(db, id).await,
            Db::SqliteDb(db) => Self::sqlitedb_delete(db, id).await,
        }
    }

    async fn scylladb_insert(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(SCYLLA_INSERT, &self.to_scylladb_model()).await?;
        Ok(())
    }

    async fn scylladb_select(db: &ScyllaDb, id: &Uuid) -> Result<CollectionScyllaModel> {
        Ok(db
            .execute(SCYLLA_SELECT, [id].as_ref())
            .await?
            .first_row_typed::<CollectionScyllaModel>()?)
    }

    async fn scylladb_select_many_by_project_id(
        db: &ScyllaDb,
        project_id: &Uuid,
    ) -> Result<ScyllaTypedRowIter<CollectionScyllaModel>> {
        Ok(db
            .execute(SCYLLA_SELECT_MANY_BY_PROJECT_ID, [project_id].as_ref())
            .await?
            .rows_typed::<CollectionScyllaModel>()?)
    }

    async fn scylladb_update(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(
            SCYLLA_UPDATE,
            &(
                &ScyllaCqlTimestamp(self.updated_at.timestamp_millis()),
                &self.name,
                &self
                    .schema_fields
                    .iter()
                    .map(|(key, value)| (key.to_owned(), value.to_scylladb_model()))
                    .collect::<HashMap<_, _>>(),
                &self.indexes,
                &self.id,
            ),
        )
        .await?;
        Ok(())
    }

    async fn scylladb_delete(db: &ScyllaDb, id: &Uuid) -> Result<()> {
        db.execute(SCYLLA_DELETE, [id].as_ref()).await?;
        Ok(())
    }

    async fn postgresdb_insert(&self, db: &PostgresDb) -> Result<()> {
        db.execute(
            sqlx::query(POSTGRES_INSERT)
                .bind(&self.id)
                .bind(&self.created_at)
                .bind(&self.updated_at)
                .bind(&self.project_id)
                .bind(&self.name)
                .bind(&serde_json::to_string(&self.schema_fields)?)
                .bind(&serde_json::to_string(&self.indexes)?),
        )
        .await?;
        Ok(())
    }

    async fn postgresdb_select(db: &PostgresDb, id: &Uuid) -> Result<CollectionPostgresModel> {
        Ok(db
            .fetch_one(sqlx::query_as(POSTGRES_SELECT).bind(id))
            .await?)
    }

    async fn postgresdb_select_many_by_project_id(
        db: &PostgresDb,
        project_id: &Uuid,
    ) -> Result<Vec<CollectionPostgresModel>> {
        Ok(db
            .fetch_all(sqlx::query_as(POSTGRES_SELECT_MANY_BY_PROJECT_ID).bind(project_id))
            .await?)
    }

    async fn postgresdb_update(&self, db: &PostgresDb) -> Result<()> {
        db.execute(
            sqlx::query(POSTGRES_UPDATE)
                .bind(&self.updated_at)
                .bind(&self.name)
                .bind(&sqlx::types::Json(&self.schema_fields))
                .bind(&sqlx::types::Json(&self.indexes))
                .bind(&self.id),
        )
        .await?;
        Ok(())
    }

    async fn postgresdb_delete(db: &PostgresDb, id: &Uuid) -> Result<()> {
        db.execute(sqlx::query(POSTGRES_DELETE).bind(id)).await?;
        Ok(())
    }

    async fn mysqldb_insert(&self, db: &MysqlDb) -> Result<()> {
        db.execute(
            sqlx::query(MYSQL_INSERT)
                .bind(&self.id)
                .bind(&self.created_at)
                .bind(&self.updated_at)
                .bind(&self.project_id)
                .bind(&self.name)
                .bind(&serde_json::to_string(&self.schema_fields)?)
                .bind(&serde_json::to_string(&self.indexes)?),
        )
        .await?;
        Ok(())
    }

    async fn mysqldb_select(db: &MysqlDb, id: &Uuid) -> Result<CollectionMysqlModel> {
        Ok(db.fetch_one(sqlx::query_as(MYSQL_SELECT).bind(id)).await?)
    }

    async fn mysqldb_select_many_by_project_id(
        db: &MysqlDb,
        project_id: &Uuid,
    ) -> Result<Vec<CollectionMysqlModel>> {
        Ok(db
            .fetch_all(sqlx::query_as(MYSQL_SELECT_MANY_BY_PROJECT_ID).bind(project_id))
            .await?)
    }

    async fn mysqldb_update(&self, db: &MysqlDb) -> Result<()> {
        db.execute(
            sqlx::query(MYSQL_UPDATE)
                .bind(&self.updated_at)
                .bind(&self.name)
                .bind(&sqlx::types::Json(&self.schema_fields))
                .bind(&sqlx::types::Json(&self.indexes))
                .bind(&self.id),
        )
        .await?;
        Ok(())
    }

    async fn mysqldb_delete(db: &MysqlDb, id: &Uuid) -> Result<()> {
        db.execute(sqlx::query(MYSQL_DELETE).bind(id)).await?;
        Ok(())
    }

    async fn sqlitedb_insert(&self, db: &SqliteDb) -> Result<()> {
        db.execute(
            sqlx::query(SQLITE_INSERT)
                .bind(&self.id)
                .bind(&self.created_at)
                .bind(&self.updated_at)
                .bind(&self.project_id)
                .bind(&self.name)
                .bind(&serde_json::to_string(&self.schema_fields)?)
                .bind(&serde_json::to_string(&self.indexes)?),
        )
        .await?;
        Ok(())
    }

    async fn sqlitedb_select(db: &SqliteDb, id: &Uuid) -> Result<CollectionSqliteModel> {
        Ok(db.fetch_one(sqlx::query_as(SQLITE_SELECT).bind(id)).await?)
    }

    async fn sqlitedb_select_many_by_project_id(
        db: &SqliteDb,
        project_id: &Uuid,
    ) -> Result<Vec<CollectionSqliteModel>> {
        Ok(db
            .fetch_all(sqlx::query_as(SQLITE_SELECT_MANY_BY_PROJECT_ID).bind(project_id))
            .await?)
    }

    async fn sqlitedb_update(&self, db: &SqliteDb) -> Result<()> {
        db.execute(
            sqlx::query(SQLITE_UPDATE)
                .bind(&self.updated_at)
                .bind(&self.name)
                .bind(&sqlx::types::Json(&self.schema_fields))
                .bind(&sqlx::types::Json(&self.indexes))
                .bind(&self.id),
        )
        .await?;
        Ok(())
    }

    async fn sqlitedb_delete(db: &SqliteDb, id: &Uuid) -> Result<()> {
        db.execute(sqlx::query(SQLITE_DELETE).bind(id)).await?;
        Ok(())
    }

    fn from_scylladb_model(model: &CollectionScyllaModel) -> Result<Self> {
        let mut schema_fields = HashMap::with_capacity(model.schema_fields().len());
        for (key, value) in model.schema_fields() {
            let value = match SchemaFieldPropsModel::from_scylladb_model(value) {
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
        )
    }

    fn from_postgresdb_model(model: &CollectionPostgresModel) -> Result<Self> {
        let mut schema_fields = HashMap::with_capacity(model.schema_fields().len());
        for (key, value) in model.schema_fields() {
            let value = match SchemaFieldPropsModel::from_postgresdb_model(value) {
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
            indexes: model.indexes().to_owned(),
            _preserve: None,
        })
    }

    fn from_mysqldb_model(model: &CollectionMysqlModel) -> Result<Self> {
        let mut schema_fields = HashMap::with_capacity(model.schema_fields().len());
        for (key, value) in model.schema_fields() {
            let value = match SchemaFieldPropsModel::from_mysqldb_model(value) {
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
            indexes: model.indexes().to_owned(),
            _preserve: None,
        })
    }

    fn from_sqlitedb_model(model: &CollectionSqliteModel) -> Result<Self> {
        let mut schema_fields = HashMap::with_capacity(model.schema_fields().len());
        for (key, value) in model.schema_fields() {
            let value = match SchemaFieldPropsModel::from_sqlitedb_model(value) {
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
            indexes: model.indexes().to_owned(),
            _preserve: None,
        })
    }
}

#[derive(Serialize, Clone, Copy)]
pub struct SchemaFieldPropsModel {
    kind: SchemaFieldKind,
    required: bool,
}

impl SchemaFieldPropsModel {
    pub fn new(kind: &SchemaFieldKind, required: &bool) -> Self {
        Self {
            kind: *kind,
            required: *required,
        }
    }

    pub fn kind(&self) -> &SchemaFieldKind {
        &self.kind
    }

    pub fn required(&self) -> &bool {
        &self.required
    }

    fn from_scylladb_model(model: &SchemaFieldPropsScyllaModel) -> Result<Self> {
        let kind = match SchemaFieldKind::from_str(model.kind()) {
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
        let kind = match SchemaFieldKind::from_str(model.kind()) {
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
        let kind = match SchemaFieldKind::from_str(model.kind()) {
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
        let kind = match SchemaFieldKind::from_str(model.kind()) {
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

#[derive(Serialize, PartialEq, Clone, Copy)]
pub enum SchemaFieldKind {
    Boolean,   // boolean
    TinyInt,   // 8-bit signed int
    SmallInt,  // 16-bit signed int
    Int,       // 32-bit signed int
    BigInt,    // 64-bit signed long
    Varint,    // Arbitrary-precision integer
    Float,     // 32-bit IEEE-754 floating point
    Double,    // 64-bit IEEE-754 floating point
    Decimal,   // Variable-precision decimal
    String,    // UTF8 encoded string
    Binary,    // Arbitrary bytes
    Uuid,      // A UUID (of any version)
    Date,      // A date (with no corresponding time value)
    Time,      // A time (with no corresponding date value)
    DateTime,  // A datetime
    Timestamp, // A timestamp (date and time)
    Json,      // A json data format
}

impl SchemaFieldKind {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Boolean => "boolean",
            Self::TinyInt => "tinyint",
            Self::SmallInt => "smallint",
            Self::Int => "int",
            Self::BigInt => "bigint",
            Self::Varint => "varint",
            Self::Float => "float",
            Self::Double => "double",
            Self::Decimal => "decimal",
            Self::String => "string",
            Self::Binary => "binary",
            Self::Uuid => "uuid",
            Self::Date => "date",
            Self::Time => "time",
            Self::DateTime => "datetime",
            Self::Timestamp => "timestamp",
            Self::Json => "json",
        }
    }

    pub fn from_str(str: &str) -> Result<Self> {
        match str {
            "boolean" => Ok(Self::Boolean),
            "tinyint" => Ok(Self::TinyInt),
            "smallint" => Ok(Self::SmallInt),
            "int" => Ok(Self::Int),
            "bigint" => Ok(Self::BigInt),
            "varint" => Ok(Self::Varint),
            "float" => Ok(Self::Float),
            "double" => Ok(Self::Double),
            "decimal" => Ok(Self::Decimal),
            "string" => Ok(Self::String),
            "binary" => Ok(Self::Binary),
            "uuid" => Ok(Self::Uuid),
            "date" => Ok(Self::Date),
            "time" => Ok(Self::Time),
            "datetime" => Ok(Self::DateTime),
            "timestamp" => Ok(Self::Timestamp),
            "json" => Ok(Self::Json),
            _ => Err(Error::msg("Unknown schema field kind")),
        }
    }

    fn to_scylladb_model(&self) -> SchemaFieldScyllaKind {
        match self {
            Self::Boolean => SchemaFieldScyllaKind::Boolean,
            Self::TinyInt => SchemaFieldScyllaKind::TinyInt,
            Self::SmallInt => SchemaFieldScyllaKind::SmallInt,
            Self::Int => SchemaFieldScyllaKind::Int,
            Self::BigInt => SchemaFieldScyllaKind::BigInt,
            Self::Varint => SchemaFieldScyllaKind::Varint,
            Self::Float => SchemaFieldScyllaKind::Float,
            Self::Double => SchemaFieldScyllaKind::Double,
            Self::Decimal => SchemaFieldScyllaKind::Decimal,
            Self::String => SchemaFieldScyllaKind::Text,
            Self::Binary | Self::Json => SchemaFieldScyllaKind::Blob,
            Self::Uuid => SchemaFieldScyllaKind::Uuid,
            Self::Date => SchemaFieldScyllaKind::Date,
            Self::Time => SchemaFieldScyllaKind::Time,
            Self::DateTime | Self::Timestamp => SchemaFieldScyllaKind::Timestamp,
        }
    }

    pub fn to_postgresdb_model(&self) -> SchemaFieldPostgresKind {
        match self {
            Self::Boolean => SchemaFieldPostgresKind::Bool,
            Self::TinyInt => SchemaFieldPostgresKind::Char,
            Self::SmallInt => SchemaFieldPostgresKind::Smallint,
            Self::Int => SchemaFieldPostgresKind::Integer,
            Self::BigInt => SchemaFieldPostgresKind::Bigint,
            Self::Varint => SchemaFieldPostgresKind::Numeric,
            Self::Float => SchemaFieldPostgresKind::Real,
            Self::Double => SchemaFieldPostgresKind::DoublePrecision,
            Self::Decimal => SchemaFieldPostgresKind::Numeric,
            Self::String => SchemaFieldPostgresKind::Varchar,
            Self::Binary => SchemaFieldPostgresKind::Bytea,
            Self::Uuid => SchemaFieldPostgresKind::Uuid,
            Self::Date => SchemaFieldPostgresKind::Date,
            Self::Time => SchemaFieldPostgresKind::Time,
            Self::DateTime => SchemaFieldPostgresKind::Timestampz,
            Self::Timestamp => SchemaFieldPostgresKind::Timestampz,
            Self::Json => SchemaFieldPostgresKind::Jsonb,
        }
    }

    fn to_mysqldb_model(&self) -> SchemaFieldMysqlKind {
        match self {
            Self::Boolean => SchemaFieldMysqlKind::Bool,
            Self::TinyInt => SchemaFieldMysqlKind::Tinyint,
            Self::SmallInt => SchemaFieldMysqlKind::Smallint,
            Self::Int => SchemaFieldMysqlKind::Int,
            Self::BigInt => SchemaFieldMysqlKind::Bigint,
            Self::Varint => SchemaFieldMysqlKind::Decimal,
            Self::Float => SchemaFieldMysqlKind::Float,
            Self::Double => SchemaFieldMysqlKind::Double,
            Self::Decimal => SchemaFieldMysqlKind::Decimal,
            Self::String => SchemaFieldMysqlKind::Varchar,
            Self::Binary => SchemaFieldMysqlKind::Blob,
            Self::Uuid => SchemaFieldMysqlKind::Binary16,
            Self::Date => SchemaFieldMysqlKind::Date,
            Self::Time => SchemaFieldMysqlKind::Time,
            Self::DateTime => SchemaFieldMysqlKind::Datetime,
            Self::Timestamp => SchemaFieldMysqlKind::Timestamp,
            Self::Json => SchemaFieldMysqlKind::Json,
        }
    }

    fn to_sqlitedb_model(&self) -> SchemaFieldSqliteKind {
        match self {
            Self::Boolean => SchemaFieldSqliteKind::Integer,
            Self::TinyInt => SchemaFieldSqliteKind::Integer,
            Self::SmallInt => SchemaFieldSqliteKind::Integer,
            Self::Int => SchemaFieldSqliteKind::Integer,
            Self::BigInt => SchemaFieldSqliteKind::Integer,
            Self::Varint => SchemaFieldSqliteKind::Text,
            Self::Float => SchemaFieldSqliteKind::Real,
            Self::Double => SchemaFieldSqliteKind::Real,
            Self::Decimal => SchemaFieldSqliteKind::Text,
            Self::String => SchemaFieldSqliteKind::Text,
            Self::Binary => SchemaFieldSqliteKind::Blob,
            Self::Uuid => SchemaFieldSqliteKind::Text,
            Self::Date => SchemaFieldSqliteKind::Text,
            Self::Time => SchemaFieldSqliteKind::Text,
            Self::DateTime => SchemaFieldSqliteKind::Text,
            Self::Timestamp => SchemaFieldSqliteKind::Text,
            Self::Json => SchemaFieldSqliteKind::Text,
        }
    }
}

struct Preserve {
    schema_fields: Option<HashMap<String, SchemaFieldPropsModel>>,
    indexes: Option<HashSet<String>>,
}
