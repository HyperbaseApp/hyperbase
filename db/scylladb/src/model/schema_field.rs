use super::schema_field_kind::SchemaScyllaFieldKind;

pub struct SchemaFieldModel {
    name: String,
    kind: SchemaScyllaFieldKind,
    required: bool,
}

impl SchemaFieldModel {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> &SchemaScyllaFieldKind {
        &self.kind
    }

    fn required(&self) -> &bool {
        &self.required
    }
}
