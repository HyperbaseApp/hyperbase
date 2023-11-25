use ahash::{HashMap, HashSet};
use scylla::{
    cql_to_rust::{FromCqlVal, FromCqlValError},
    frame::{
        response::result::CqlValue,
        value::{Timestamp, Value, ValueTooBig},
    },
    BufMut, FromRow, FromUserType, IntoUserType, ValueList,
};
use uuid::Uuid;

#[derive(ValueList, FromRow)]
pub struct CollectionScyllaModel {
    id: Uuid,
    created_at: Timestamp,
    updated_at: Timestamp,
    project_id: Uuid,
    name: String,
    schema_fields: HashMap<String, SchemaFieldPropsScyllaModel>,
    indexes: Option<HashSet<String>>,
}

impl CollectionScyllaModel {
    pub fn new(
        id: &Uuid,
        created_at: &Timestamp,
        updated_at: &Timestamp,
        project_id: &Uuid,
        name: &str,
        schema_fields: &HashMap<String, SchemaFieldPropsScyllaModel>,
        indexes: &Option<HashSet<String>>,
    ) -> Self {
        Self {
            id: *id,
            created_at: *created_at,
            updated_at: *updated_at,
            project_id: *project_id,
            name: name.to_owned(),
            schema_fields: schema_fields.clone(),
            indexes: indexes.clone(),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    pub fn updated_at(&self) -> &Timestamp {
        &self.updated_at
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn schema_fields(&self) -> &HashMap<String, SchemaFieldPropsScyllaModel> {
        &self.schema_fields
    }

    pub fn indexes(&self) -> &Option<HashSet<String>> {
        &self.indexes
    }
}

#[derive(IntoUserType, FromUserType, Clone)]
pub struct SchemaFieldPropsScyllaModel {
    kind: SchemaFieldScyllaKind,
    required: bool,
}

impl SchemaFieldPropsScyllaModel {
    pub fn new(kind: &SchemaFieldScyllaKind, required: &bool) -> Self {
        Self {
            kind: *kind,
            required: *required,
        }
    }

    pub fn kind(&self) -> &SchemaFieldScyllaKind {
        &self.kind
    }

    pub fn required(&self) -> &bool {
        &self.required
    }
}

#[derive(Clone, Copy)]
pub enum SchemaFieldScyllaKind {
    Boolean,
    Tinyint,
    Smallint,
    Int,
    Bigint,
    Float,
    Double,
    Ascii,
    Text,
    Varchar,
    Blob,
    Inet,
    Uuid,
    Timeuuid,
    Date,
    Time,
    Timestamp,
    Duration,
    Decimal,
    Varint,
    List,
    Set,
    Map,
    Tuple,
}

impl SchemaFieldScyllaKind {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Boolean => "boolean",
            Self::Tinyint => "tinyint",
            Self::Smallint => "smallint",
            Self::Int => "int",
            Self::Bigint => "bigint",
            Self::Float => "float",
            Self::Double => "double",
            Self::Ascii => "ascii",
            Self::Text => "text",
            Self::Varchar => "varchar",
            Self::Blob => "blob",
            Self::Inet => "inet",
            Self::Uuid => "uuid",
            Self::Timeuuid => "timeuuid",
            Self::Date => "date",
            Self::Time => "time",
            Self::Timestamp => "timestamp",
            Self::Duration => "duration",
            Self::Decimal => "decimal",
            Self::Varint => "varint",
            Self::List => "list",
            Self::Set => "set",
            Self::Map => "map",
            Self::Tuple => "tuple",
        }
    }

    pub fn from_str(str: &str) -> Result<Self, &str> {
        match str {
            "boolean" => Ok(Self::Boolean),
            "tinyint" => Ok(Self::Tinyint),
            "smallint" => Ok(Self::Smallint),
            "int" => Ok(Self::Int),
            "bigint" => Ok(Self::Bigint),
            "float" => Ok(Self::Float),
            "double" => Ok(Self::Double),
            "ascii" => Ok(Self::Ascii),
            "text" => Ok(Self::Text),
            "varchar" => Ok(Self::Varchar),
            "blob" => Ok(Self::Blob),
            "inet" => Ok(Self::Inet),
            "uuid" => Ok(Self::Uuid),
            "timeuuid" => Ok(Self::Timeuuid),
            "date" => Ok(Self::Date),
            "time" => Ok(Self::Time),
            "timestamp" => Ok(Self::Timestamp),
            "duration" => Ok(Self::Duration),
            "decimal" => Ok(Self::Decimal),
            "varint" => Ok(Self::Varint),
            "list" => Ok(Self::List),
            "set" => Ok(Self::Set),
            "map" => Ok(Self::Map),
            "tuple" => Ok(Self::Tuple),
            _ => Err("Unknown schema field kind"),
        }
    }
}

impl Value for SchemaFieldScyllaKind {
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), ValueTooBig> {
        let kind = self.to_str();
        let str_bytes: &[u8] = kind.as_bytes();
        let val_len: i32 = str_bytes.len().try_into().map_err(|_| ValueTooBig)?;

        buf.put_i32(val_len);
        buf.put(str_bytes);

        Ok(())
    }
}

impl FromCqlVal<CqlValue> for SchemaFieldScyllaKind {
    fn from_cql(cql_val: CqlValue) -> Result<Self, FromCqlValError> {
        Ok(Self::from_str(&cql_val.as_text().unwrap()).unwrap())
    }
}
