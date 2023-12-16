use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use futures::future;
use hb_db_scylladb::{
    db::ScyllaDb,
    model::{
        collection::{CollectionScyllaModel, SchemaFieldPropsScyllaModel},
        system::SchemaFieldScyllaKind,
    },
    query::collection::{DELETE, INSERT, SELECT, SELECT_MANY_BY_PROJECT_ID, UPDATE},
};
use scylla::{frame::value::Timestamp, transport::session::TypedRowIter};
use uuid::Uuid;

use crate::{
    record::RecordDao,
    util::conversion::{datetime_to_duration_since_epoch, duration_since_epoch_to_datetime},
    Db,
};

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
        }
    }

    pub async fn db_select(db: &Db, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Ok(Self::from_scylladb_model(
                &Self::scylladb_select(db, id).await?,
            )?),
        }
    }

    pub async fn db_select_many_by_project_id(db: &Db, project_id: &Uuid) -> Result<Vec<Self>> {
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
        }
    }

    pub async fn db_delete(db: &Db, id: &Uuid) -> Result<()> {
        RecordDao::db_drop_table(db, id).await?;

        match db {
            Db::ScyllaDb(db) => Self::scylladb_delete(db, id).await,
        }
    }

    async fn scylladb_insert(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(INSERT, &self.to_scylladb_model()).await?;
        Ok(())
    }

    async fn scylladb_select(db: &ScyllaDb, id: &Uuid) -> Result<CollectionScyllaModel> {
        Ok(db
            .execute(SELECT, [id].as_ref())
            .await?
            .first_row_typed::<CollectionScyllaModel>()?)
    }

    async fn scylladb_select_many_by_project_id(
        db: &ScyllaDb,
        project_id: &Uuid,
    ) -> Result<TypedRowIter<CollectionScyllaModel>> {
        Ok(db
            .execute(SELECT_MANY_BY_PROJECT_ID, [project_id].as_ref())
            .await?
            .rows_typed::<CollectionScyllaModel>()?)
    }

    async fn scylladb_update(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(
            UPDATE,
            &(
                &self.updated_at,
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
        db.execute(DELETE, [id].as_ref()).await?;
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
            created_at: duration_since_epoch_to_datetime(&model.created_at().0)?,
            updated_at: duration_since_epoch_to_datetime(&model.updated_at().0)?,
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
            &Timestamp(datetime_to_duration_since_epoch(&self.created_at)),
            &Timestamp(datetime_to_duration_since_epoch(&self.updated_at)),
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
}

#[derive(Clone, Copy)]
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
}

#[derive(PartialEq, Clone, Copy)]
pub enum SchemaFieldKind {
    Bool,      // boolean
    TinyInt,   // 8-bit signed int
    SmallInt,  // 16-bit signed int
    Int,       // 32-bit signed int
    BigInt,    // 64-bit signed long
    Float,     // 32-bit IEEE-754 floating point
    Double,    // 64-bit IEEE-754 floating point
    String,    // UTF8 encoded string
    Bytes,     // Arbitrary bytes
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
            Self::Bool => "bool",
            Self::TinyInt => "tiny_int",
            Self::SmallInt => "small_int",
            Self::Int => "int",
            Self::BigInt => "big_int",
            Self::Float => "float",
            Self::Double => "double",
            Self::String => "string",
            Self::Bytes => "byte",
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
            "bool" => Ok(Self::Bool),
            "tiny_int" => Ok(Self::TinyInt),
            "small_int" => Ok(Self::SmallInt),
            "int" => Ok(Self::Int),
            "big_int" => Ok(Self::BigInt),
            "float" => Ok(Self::Float),
            "double" => Ok(Self::Double),
            "string" => Ok(Self::String),
            "bytes" => Ok(Self::Bytes),
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
            Self::Bool => SchemaFieldScyllaKind::Boolean,
            Self::TinyInt => SchemaFieldScyllaKind::Tinyint,
            Self::SmallInt => SchemaFieldScyllaKind::Smallint,
            Self::Int => SchemaFieldScyllaKind::Int,
            Self::BigInt => SchemaFieldScyllaKind::Bigint,
            Self::Float => SchemaFieldScyllaKind::Float,
            Self::Double => SchemaFieldScyllaKind::Double,
            Self::String => SchemaFieldScyllaKind::Text,
            Self::Bytes | Self::Json => SchemaFieldScyllaKind::Blob,
            Self::Uuid => SchemaFieldScyllaKind::Uuid,
            Self::Date => SchemaFieldScyllaKind::Date,
            Self::Time => SchemaFieldScyllaKind::Time,
            Self::DateTime | Self::Timestamp => SchemaFieldScyllaKind::Timestamp,
        }
    }
}

struct Preserve {
    schema_fields: Option<HashMap<String, SchemaFieldPropsModel>>,
    indexes: Option<HashSet<String>>,
}
