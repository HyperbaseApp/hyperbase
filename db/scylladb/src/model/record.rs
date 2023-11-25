use ahash::{HashMap, HashMapExt};
use scylla::frame::response::result::CqlValue;

pub struct RecordScyllaModel<'a> {
    data: HashMap<&'a str, CqlValue>,
}

impl RecordScyllaModel<'_> {
    pub fn new(capacity: &Option<usize>) -> Self {
        match capacity {
            Some(capacity) => Self {
                data: HashMap::with_capacity(*capacity),
            },
            None => Self {
                data: HashMap::new(),
            },
        }
    }

    pub fn get(&self, key: &str) -> Option<&CqlValue> {
        self.data.get(key)
    }
}
