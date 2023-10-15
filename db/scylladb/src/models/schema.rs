use hb_db::DbRecordSchemaV;

pub struct ScyllaRecordSchemaV<'a> {
    kind: &'a str,
    value: Box<dyn std::any::Any>,
}

impl<'a> DbRecordSchemaV for ScyllaRecordSchemaV<'a> {
    fn kind(&self) -> &str {
        self.kind
    }

    fn value(&self) -> &Box<dyn std::any::Any> {
        &self.value
    }
}
