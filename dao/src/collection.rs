use std::collections::HashMap;

use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_scylladb::{
    db::ScyllaDb,
    model::collection::{CollectionScyllaModel, SchemaFieldScyllaKind, SchemaFieldScyllaModel},
};
use scylla::{frame::value::Timestamp, transport::session::TypedRowIter};
use strum::{Display, EnumString};
use uuid::Uuid;

use crate::{
    util::conversion::{datetime_to_duration_since_epoch, duration_since_epoch_to_datetime},
    Db,
};

pub struct CollectionDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    project_id: Uuid,
    name: String,
    schema_fields: HashMap<String, SchemaFieldModel>,
    indexes: Vec<String>,
}

impl CollectionDao {
    pub fn new(
        project_id: &Uuid,
        name: &str,
        schema_fields: &HashMap<String, SchemaFieldModel>,
        indexes: &Vec<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            project_id: *project_id,
            name: name.to_string(),
            schema_fields: schema_fields.to_owned(),
            indexes: indexes.to_vec(),
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

    pub fn schema_fields(&self) -> &HashMap<String, SchemaFieldModel> {
        &self.schema_fields
    }

    pub fn indexes(&self) -> &Vec<String> {
        &self.indexes
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn set_schema_fields(&mut self, schema_fields: &HashMap<String, SchemaFieldModel>) {
        self.schema_fields = schema_fields.to_owned();
    }

    pub fn set_indexes(&mut self, indexes: &Vec<String>) {
        self.indexes = indexes.to_vec();
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_insert(&self, db).await,
        }
    }

    pub async fn db_select(db: &Db, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Ok(Self::from_scylladb_model(
                &Self::scylladb_select(db, id).await?,
            )?),
        }
    }

    pub async fn db_select_by_project_id(db: &Db, project_id: &Uuid) -> Result<Vec<Self>> {
        match db {
            Db::ScyllaDb(db) => {
                let mut collections_data = Vec::new();
                let collections = Self::scylladb_select_many_by_project_id(db, project_id).await?;
                for collection in collections {
                    if let Ok(model) = &collection {
                        collections_data.push(Self::from_scylladb_model(model)?)
                    } else if let Err(err) = collection {
                        return Err(err.into());
                    }
                }
                Ok(collections_data)
            }
        }
    }

    pub async fn db_update(&mut self, db: &Db) -> Result<()> {
        self.updated_at = Utc::now();
        match db {
            Db::ScyllaDb(db) => Self::scylladb_update(&self, db).await,
        }
    }

    pub async fn db_delete(db: &Db, id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_delete(db, id).await,
        }
    }

    async fn scylladb_insert(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(
            db.prepared_statement().collection().insert(),
            self.to_scylladb_model(),
        )
        .await?;
        Ok(())
    }

    async fn scylladb_select(db: &ScyllaDb, id: &Uuid) -> Result<CollectionScyllaModel> {
        Ok(db
            .execute(db.prepared_statement().collection().select(), [id].as_ref())
            .await?
            .first_row_typed::<CollectionScyllaModel>()?)
    }

    async fn scylladb_select_many_by_project_id(
        db: &ScyllaDb,
        project_id: &Uuid,
    ) -> Result<TypedRowIter<CollectionScyllaModel>> {
        Ok(db
            .execute(
                db.prepared_statement()
                    .collection()
                    .select_many_by_project_id(),
                [project_id].as_ref(),
            )
            .await?
            .rows_typed::<CollectionScyllaModel>()?)
    }

    async fn scylladb_update(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(
            db.prepared_statement().collection().update(),
            (
                &self.updated_at,
                &self.name,
                &self
                    .schema_fields
                    .iter()
                    .map(|(key, value)| (key.to_owned(), value.to_scylladb_model().to_owned()))
                    .collect::<HashMap<_, _>>(),
                &self.indexes,
                &self.id,
            ),
        )
        .await?;
        Ok(())
    }

    async fn scylladb_delete(db: &ScyllaDb, id: &Uuid) -> Result<()> {
        db.execute(db.prepared_statement().collection().delete(), [id].as_ref())
            .await?;
        Ok(())
    }

    fn from_scylladb_model(model: &CollectionScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: duration_since_epoch_to_datetime(model.created_at().0)?,
            updated_at: duration_since_epoch_to_datetime(model.updated_at().0)?,
            project_id: *model.project_id(),
            name: model.name().to_string(),
            schema_fields: model
                .schema_fields()
                .iter()
                .map(|(key, value)| (key.to_owned(), SchemaFieldModel::from_scylladb_model(value)))
                .collect(),
            indexes: model.indexes().to_vec(),
        })
    }

    fn to_scylladb_model(&self) -> CollectionScyllaModel {
        CollectionScyllaModel::new(
            &self.id,
            &Timestamp(datetime_to_duration_since_epoch(self.created_at)),
            &Timestamp(datetime_to_duration_since_epoch(self.updated_at)),
            &self.project_id,
            &self.name,
            &self
                .schema_fields
                .iter()
                .map(|(key, value)| (key.to_owned(), value.to_scylladb_model().to_owned()))
                .collect(),
            &self.indexes,
        )
    }
}

#[derive(Clone, Copy)]
pub struct SchemaFieldModel {
    kind: SchemaFieldKind,
    required: bool,
}

impl SchemaFieldModel {
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

    fn from_scylladb_model(model: &SchemaFieldScyllaModel) -> Self {
        Self {
            kind: SchemaFieldKind::from_scylladb_model(model.kind()),
            required: *model.required(),
        }
    }

    fn to_scylladb_model(self) -> SchemaFieldScyllaModel {
        SchemaFieldScyllaModel::new(&self.kind.to_scylladb_model(), &self.required)
    }
}

#[derive(EnumString, Display, Clone, Copy)]
pub enum SchemaFieldKind {
    Boolean,      // boolean
    TinyInteger,  // 8-bit signed int
    SmallInteger, // 16-bit signed int
    Integer,      // 32-bit signed int
    BigInteger,   // 64-bit signed long
    Float,        // 32-bit IEEE-754 floating point
    Double,       // 64-bit IEEE-754 floating point
    String,       // UTF8 encoded string
    Byte,         // Arbitrary bytes
    Uuid,         // A UUID (of any version)
    Date,         // A date (with no corresponding time value)
    Time,         // A time (with no corresponding date value)
    DateTime,     // A datetime
    Timestamp,    // A timestamp (date and time)
    Json,         // A json data format
}

impl SchemaFieldKind {
    fn from_scylladb_model(model: &SchemaFieldScyllaKind) -> Self {
        match model {
            SchemaFieldScyllaKind::Boolean => Self::Boolean,
            SchemaFieldScyllaKind::Tinyint => Self::TinyInteger,
            SchemaFieldScyllaKind::Smallint => Self::SmallInteger,
            SchemaFieldScyllaKind::Int => Self::Integer,
            SchemaFieldScyllaKind::Bigint | SchemaFieldScyllaKind::Varint => Self::BigInteger,
            SchemaFieldScyllaKind::Float => Self::Float,
            SchemaFieldScyllaKind::Double | SchemaFieldScyllaKind::Decimal => Self::Double,
            SchemaFieldScyllaKind::Ascii
            | SchemaFieldScyllaKind::Text
            | SchemaFieldScyllaKind::Inet
            | SchemaFieldScyllaKind::Varchar => Self::String,
            SchemaFieldScyllaKind::Blob => Self::Byte,
            SchemaFieldScyllaKind::Uuid => Self::Uuid,
            SchemaFieldScyllaKind::Timeuuid => Self::Uuid,
            SchemaFieldScyllaKind::Date => Self::Date,
            SchemaFieldScyllaKind::Time => Self::Time,
            SchemaFieldScyllaKind::Timestamp | SchemaFieldScyllaKind::Duration => Self::Timestamp,
            SchemaFieldScyllaKind::List
            | SchemaFieldScyllaKind::Set
            | SchemaFieldScyllaKind::Map
            | SchemaFieldScyllaKind::Tuple => Self::Byte,
        }
    }

    fn to_scylladb_model(&self) -> SchemaFieldScyllaKind {
        match self {
            Self::Boolean => SchemaFieldScyllaKind::Boolean,
            Self::TinyInteger => SchemaFieldScyllaKind::Tinyint,
            Self::SmallInteger => SchemaFieldScyllaKind::Smallint,
            Self::Integer => SchemaFieldScyllaKind::Int,
            Self::BigInteger => SchemaFieldScyllaKind::Bigint,
            Self::Float => SchemaFieldScyllaKind::Float,
            Self::Double => SchemaFieldScyllaKind::Double,
            Self::String => SchemaFieldScyllaKind::Text,
            Self::Byte | Self::Json => SchemaFieldScyllaKind::Blob,
            Self::Uuid => SchemaFieldScyllaKind::Uuid,
            Self::Date => SchemaFieldScyllaKind::Date,
            Self::Time => SchemaFieldScyllaKind::Time,
            Self::DateTime | Self::Timestamp => SchemaFieldScyllaKind::Timestamp,
        }
    }
}
