use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_scylladb::{
    db::ScyllaDb,
    model::collection::{CollectionScyllaModel, SchemaScyllaFieldKind, SchemaScyllaFieldModel},
};
use scylla::frame::value::Timestamp;
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
    schema_fields: Vec<SchemaFieldModel>,
    indexes: Vec<String>,
}

impl CollectionDao {
    pub fn new(
        project_id: &Uuid,
        name: &str,
        schema_fields: &Vec<SchemaFieldModel>,
        indexes: &Vec<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            project_id: *project_id,
            name: name.to_string(),
            schema_fields: schema_fields.to_vec(),
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

    pub fn schema_fields(&self) -> &Vec<SchemaFieldModel> {
        &self.schema_fields
    }

    pub fn indexes(&self) -> &Vec<String> {
        &self.indexes
    }
}

impl CollectionDao {
    pub async fn insert(&self, db: &Db<'_>) -> Result<()> {
        match *db {
            Db::ScyllaDb(db) => Self::scylladb_insert(&self, db).await,
        }
    }

    pub async fn select(db: &Db<'_>, id: &Uuid) -> Result<Self> {
        match *db {
            Db::ScyllaDb(db) => Ok(Self::from_scylladb_model(
                &Self::scylladb_select(db, id).await?,
            )?),
        }
    }
}

impl CollectionDao {
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
                .map(|field| SchemaFieldModel::from_scylladb_model(field))
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
                .clone()
                .into_iter()
                .map(|schema_field| schema_field.to_scylladb_model())
                .collect(),
            &self.indexes,
        )
    }
}

#[derive(Clone)]
pub struct SchemaFieldModel {
    name: String,
    kind: SchemaFieldKind,
    required: bool,
}

impl SchemaFieldModel {
    fn from_scylladb_model(model: &SchemaScyllaFieldModel) -> Self {
        Self {
            name: model.name().to_string(),
            kind: SchemaFieldKind::from_scylladb_model(model.kind()),
            required: *model.required(),
        }
    }

    fn to_scylladb_model(self) -> SchemaScyllaFieldModel {
        SchemaScyllaFieldModel::new(&self.name, &self.kind.to_scylladb_model(), &self.required)
    }
}

#[derive(Clone)]
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
    fn from_scylladb_model(model: &SchemaScyllaFieldKind) -> Self {
        match model {
            SchemaScyllaFieldKind::Boolean => Self::Boolean,
            SchemaScyllaFieldKind::Tinyint => Self::TinyInteger,
            SchemaScyllaFieldKind::Smallint => Self::SmallInteger,
            SchemaScyllaFieldKind::Int => Self::Integer,
            SchemaScyllaFieldKind::Bigint | SchemaScyllaFieldKind::Varint => Self::BigInteger,
            SchemaScyllaFieldKind::Float => Self::Float,
            SchemaScyllaFieldKind::Double | SchemaScyllaFieldKind::Decimal => Self::Double,
            SchemaScyllaFieldKind::Ascii
            | SchemaScyllaFieldKind::Text
            | SchemaScyllaFieldKind::Inet
            | SchemaScyllaFieldKind::Varchar => Self::String,
            SchemaScyllaFieldKind::Blob => Self::Byte,
            SchemaScyllaFieldKind::Uuid => Self::Uuid,
            SchemaScyllaFieldKind::Timeuuid => Self::Uuid,
            SchemaScyllaFieldKind::Date => Self::Date,
            SchemaScyllaFieldKind::Time => Self::Time,
            SchemaScyllaFieldKind::Timestamp | SchemaScyllaFieldKind::Duration => Self::Timestamp,
            SchemaScyllaFieldKind::List
            | SchemaScyllaFieldKind::Set
            | SchemaScyllaFieldKind::Map
            | SchemaScyllaFieldKind::Tuple => Self::Byte,
        }
    }

    fn to_scylladb_model(&self) -> SchemaScyllaFieldKind {
        match self {
            Self::Boolean => SchemaScyllaFieldKind::Boolean,
            Self::TinyInteger => SchemaScyllaFieldKind::Tinyint,
            Self::SmallInteger => SchemaScyllaFieldKind::Smallint,
            Self::Integer => SchemaScyllaFieldKind::Int,
            Self::BigInteger => SchemaScyllaFieldKind::Bigint,
            Self::Float => SchemaScyllaFieldKind::Float,
            Self::Double => SchemaScyllaFieldKind::Double,
            Self::String => SchemaScyllaFieldKind::Text,
            Self::Byte | Self::Json => SchemaScyllaFieldKind::Blob,
            Self::Uuid => SchemaScyllaFieldKind::Uuid,
            Self::Date => SchemaScyllaFieldKind::Date,
            Self::Time => SchemaScyllaFieldKind::Time,
            Self::DateTime | Self::Timestamp => SchemaScyllaFieldKind::Timestamp,
        }
    }
}
