use ahash::HashMap;
use serde_json::Value;

pub type InsertOneRecordReqJson = HashMap<String, Value>;

pub type UpdateOneRecordReqJson = HashMap<String, Value>;
