use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use hb_db_scylladb::{
    db::ScyllaDb,
    model::collection::{
        CollectionScyllaModel, SchemaFieldPropsScyllaModel, SchemaFieldScyllaKind,
    },
    query::{
        collection::{DELETE, INSERT, SELECT, SELECT_MANY_BY_PROJECT_ID, UPDATE},
        record,
    },
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

    pub fn set_schema_fields(&mut self, schema_fields: &HashMap<String, SchemaFieldPropsModel>) {
        if self._preserve.is_none() {
            self._preserve = Some(Preserve {
                schema_fields: Some(self.schema_fields.clone()),
                indexes: None,
            });
        } else {
            let preserve = self._preserve.as_mut().unwrap();
            if preserve.schema_fields.is_none() {
                preserve.schema_fields = Some(self.schema_fields.clone());
            }
        }
        self.schema_fields = schema_fields.clone();
    }

    pub fn set_indexes(&mut self, indexes: &HashSet<String>) {
        if self._preserve.is_none() {
            self._preserve = Some(Preserve {
                schema_fields: None,
                indexes: Some(self.indexes.clone()),
            });
        } else {
            let preserve = self._preserve.as_mut().unwrap();
            if preserve.indexes.is_none() {
                preserve.indexes = Some(self.indexes.clone());
            }
        }
        self.indexes = indexes.to_owned();
    }

    pub fn to_record(&self, capacity: &Option<usize>) -> RecordDao {
        RecordDao::new(&RecordDao::new_table_name(&self.id), capacity)
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
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
        self.updated_at = Utc::now();
        match db {
            Db::ScyllaDb(db) => Self::scylladb_update(self, db).await,
        }
    }

    pub async fn db_delete(db: &Db, id: &Uuid) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_delete(db, id).await,
        }
    }

    async fn scylladb_insert(&self, db: &ScyllaDb) -> Result<()> {
        let record_table = RecordDao::new_table_name(&self.id);
        db.session_query(
            record::create_table(
                &record_table,
                &self
                    .schema_fields
                    .iter()
                    .map(|(key, value)| (key.to_owned(), value.to_scylladb_model()))
                    .collect::<HashMap<_, _>>(),
            )
            .as_str(),
            &[],
        )
        .await?;
        for index in &self.indexes {
            db.session_query(record::create_index(&record_table, &index).as_str(), &[])
                .await?;
        }

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
        let record_table = RecordDao::new_table_name(&self.id);

        let is_preserve_schema_fields_exist = self
            ._preserve
            .as_ref()
            .is_some_and(|preserve| preserve.schema_fields.as_ref().is_some());
        let is_preserve_indexes_exist = self
            ._preserve
            .as_ref()
            .is_some_and(|preserve| preserve.indexes.as_ref().is_some());

        if is_preserve_indexes_exist {
            for index in self._preserve.as_ref().unwrap().indexes.as_ref().unwrap() {
                if !self.indexes.contains(index) {
                    db.session_query(&record::drop_index(&record_table, index.as_str()), &[])
                        .await?;
                }
            }
        }

        if is_preserve_schema_fields_exist {
            for field_name in self
                ._preserve
                .as_ref()
                .unwrap()
                .schema_fields
                .as_ref()
                .unwrap()
                .keys()
            {
                let mut drop_columns = HashSet::new();
                if !self.schema_fields.contains_key(field_name) {
                    drop_columns.insert(field_name.clone());
                }
                if !drop_columns.is_empty() {
                    db.session_query(&record::drop_columns(&record_table, &drop_columns), &[])
                        .await?;
                }
            }

            for (field_name, field_props) in &self.schema_fields {
                let mut add_colums = HashMap::new();
                if !self
                    ._preserve
                    .as_ref()
                    .unwrap()
                    .schema_fields
                    .as_ref()
                    .unwrap()
                    .contains_key(field_name)
                {
                    add_colums.insert(field_name.to_owned(), field_props.to_scylladb_model());
                }
                if !add_colums.is_empty() {
                    db.session_query(&record::add_columns(&record_table, &add_colums), &[])
                        .await?;
                }
            }
        }

        if is_preserve_indexes_exist {
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
                    db.session_query(&record::create_index(&record_table, index.as_str()), &[])
                        .await?;
                }
            }
        }

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
        db.session_query(
            record::drop_table(&RecordDao::new_table_name(id)).as_str(),
            &[],
        )
        .await?;

        db.execute(DELETE, [id].as_ref()).await?;

        Ok(())
    }

    fn from_scylladb_model(model: &CollectionScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: duration_since_epoch_to_datetime(&model.created_at().0)?,
            updated_at: duration_since_epoch_to_datetime(&model.updated_at().0)?,
            project_id: *model.project_id(),
            name: model.name().to_owned(),
            schema_fields: model
                .schema_fields()
                .iter()
                .map(|(key, value)| {
                    (
                        key.to_owned(),
                        SchemaFieldPropsModel::from_scylladb_model(value),
                    )
                })
                .collect(),
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

#[derive(Clone)]
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

    fn from_scylladb_model(model: &SchemaFieldPropsScyllaModel) -> Self {
        Self {
            kind: SchemaFieldKind::from_scylladb_model(model.kind()),
            required: *model.required(),
        }
    }

    fn to_scylladb_model(&self) -> SchemaFieldPropsScyllaModel {
        SchemaFieldPropsScyllaModel::new(&self.kind.to_scylladb_model(), &self.required)
    }
}

#[derive(Clone, Copy)]
pub enum SchemaFieldKind {
    Bool,      // boolean
    TinyInt,   // 8-bit signed int
    SmallInt,  // 16-bit signed int
    Int,       // 32-bit signed int
    BigInt,    // 64-bit signed long
    Float,     // 32-bit IEEE-754 floating point
    Double,    // 64-bit IEEE-754 floating point
    String,    // UTF8 encoded string
    Byte,      // Arbitrary bytes
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
            Self::Byte => "byte",
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
            "byte" => Ok(Self::Byte),
            "uuid" => Ok(Self::Uuid),
            "date" => Ok(Self::Date),
            "time" => Ok(Self::Time),
            "datetime" => Ok(Self::DateTime),
            "timestamp" => Ok(Self::Timestamp),
            "json" => Ok(Self::Json),
            _ => Err(Error::msg("Unknown schema field kind")),
        }
    }

    fn from_scylladb_model(model: &SchemaFieldScyllaKind) -> Self {
        match model {
            SchemaFieldScyllaKind::Boolean => Self::Bool,
            SchemaFieldScyllaKind::Tinyint => Self::TinyInt,
            SchemaFieldScyllaKind::Smallint => Self::SmallInt,
            SchemaFieldScyllaKind::Int => Self::Int,
            SchemaFieldScyllaKind::Bigint | SchemaFieldScyllaKind::Varint => Self::BigInt,
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
            Self::Bool => SchemaFieldScyllaKind::Boolean,
            Self::TinyInt => SchemaFieldScyllaKind::Tinyint,
            Self::SmallInt => SchemaFieldScyllaKind::Smallint,
            Self::Int => SchemaFieldScyllaKind::Int,
            Self::BigInt => SchemaFieldScyllaKind::Bigint,
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

struct Preserve {
    schema_fields: Option<HashMap<String, SchemaFieldPropsModel>>,
    indexes: Option<HashSet<String>>,
}
