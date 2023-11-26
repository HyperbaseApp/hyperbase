use ahash::{HashMap, HashSet};
use itertools::Itertools;

use crate::model::collection::SchemaFieldPropsScyllaModel;

pub const COUNT_TABLE: &str = "SELECT COUNT(1) FROM \"system_schema\".\"tables\" WHERE \"keyspace_name\" = 'hyperbase' AND \"table_name\" = ?";

pub fn create_table(
    record_table: &str,
    columns: &HashMap<String, SchemaFieldPropsScyllaModel>,
) -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS \"hyperbase\".\"{}\" (\"_id\" uuid, {}, PRIMARY KEY (\"_id\")) ",
        record_table,
        columns
            .iter()
            .map(|(col, col_props)| format!("\"{}\" {}", col, col_props.kind().to_str()))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub fn drop_table(record_table: &str) -> String {
    format!("DROP TABLE IF EXISTS \"hyperbase\".\"{record_table}\"")
}

pub fn create_index(record_table: &str, index: &str) -> String {
    format!("CREATE INDEX IF NOT EXISTS \"{record_table}_{index}\" ON \"hyperbase\".\"{record_table}\" (\"{index}\")")
}

pub fn drop_index(record_table: &str, index: &str) -> String {
    format!("DROP INDEX IF EXISTS \"hyperbase\".\"{record_table}_{index}\"")
}

pub fn add_columns(
    record_table: &str,
    columns: &HashMap<String, SchemaFieldPropsScyllaModel>,
) -> String {
    format!(
        "ALTER TABLE \"hyperbase\".\"{}\" ADD {}",
        record_table,
        columns
            .iter()
            .map(|(col, col_props)| format!("\"{}\" {}", col, col_props.kind().to_str()))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub fn drop_columns(record_table: &str, column_names: &HashSet<String>) -> String {
    format!(
        "ALTER TABLE \"hyperbase\".\"{}\" DROP {}",
        record_table,
        &column_names.iter().join(", ")
    )
}
