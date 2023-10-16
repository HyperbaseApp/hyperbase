use hb_db::model::{SchemaFieldKind as DbSchemaFieldKind, SchemaFieldModel as DbSchemaFieldModel};

use super::schema_field_kind::SchemaFieldKind;

pub struct SchemaFieldModel {
    name: String,
    kind: SchemaFieldKind,
    required: bool,
}

impl DbSchemaFieldModel for SchemaFieldModel {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> &dyn DbSchemaFieldKind {
        &self.kind
    }

    fn required(&self) -> &bool {
        &self.required
    }
}
