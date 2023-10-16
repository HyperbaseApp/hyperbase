use super::schema_field_kind::SchemaFieldKind;

pub trait SchemaFieldModel {
    fn name(&self) -> &str;
    fn kind(&self) -> &dyn SchemaFieldKind;
    fn required(&self) -> &bool;
}
