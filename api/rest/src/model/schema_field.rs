use serde::Deserialize;

#[derive(Deserialize)]
pub struct SchemaFieldModel {
    pub name: String,
    pub kind: String,
    pub required: bool,
}
