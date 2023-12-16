use ahash::{HashMap, HashSet};
use itertools::Itertools;

use crate::model::collection::SchemaFieldPropsScyllaModel;

pub fn create_table(
    record_table: &str,
    columns: &HashMap<String, SchemaFieldPropsScyllaModel>,
) -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS \"hyperbase\".\"{}\" (\"_id\" uuid, {}, PRIMARY KEY (\"_id\")) ",
        record_table,
        columns
            .iter()
            .map(|(col, col_props)| format!("\"{}\" {}", col, col_props.internal_kind().to_str()))
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
        "ALTER TABLE \"hyperbase\".\"{}\" ADD ({})",
        record_table,
        columns
            .iter()
            .map(|(col, col_props)| format!("\"{}\" {}", col, col_props.internal_kind().to_str()))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub fn drop_columns(record_table: &str, column_names: &HashSet<String>) -> String {
    format!(
        "ALTER TABLE \"hyperbase\".\"{}\" DROP ({})",
        record_table,
        &column_names
            .iter()
            .map(|col| format!("\"{col}\""))
            .join(", ")
    )
}

pub fn change_columns_type(
    record_table: &str,
    columns: &HashMap<String, SchemaFieldPropsScyllaModel>,
) -> String {
    format!(
        "ALTER TABLE \"hyperbase\".\"{}\" {}",
        record_table,
        columns
            .iter()
            .map(|(col, col_props)| format!(
                "ALTER \"{}\" TYPE {}",
                col,
                col_props.internal_kind().to_str()
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub fn insert(record_table: &str, columns: &Vec<String>) -> String {
    let mut cols = "".to_owned();
    let mut vals = "".to_owned();
    for (idx, col) in columns.iter().enumerate() {
        cols += &format!("\"{col}\"");
        vals += "?";
        if idx < columns.len() - 1 {
            cols += ", ";
            vals += ", ";
        }
    }
    format!("INSERT INTO \"hyperbase\".\"{record_table}\" ({cols}) VALUES ({vals})")
}

pub fn select(record_table: &str, columns: &Vec<String>) -> String {
    format!(
        "SELECT {} FROM \"hyperbase\".\"{}\" WHERE \"_id\" = ?",
        columns.iter().map(|col| format!("\"{col}\"")).join(", "),
        record_table
    )
}

pub fn update(record_table: &str, columns: &Vec<String>) -> String {
    format!(
        "UPDATE \"hyperbase\".\"{}\" SET {} WHERE \"_id\" = ?",
        record_table,
        columns
            .iter()
            .map(|col| format!("\"{col}\" = ?"))
            .join(", ")
    )
}

pub fn delete(record_table: &str, columns: &HashSet<String>) -> String {
    format!(
        "DELETE FROM \"hyperbase\".\"{}\" WHERE {}",
        record_table,
        columns
            .iter()
            .map(|col| format!("\"{col}\" = ?"))
            .join(", ")
    )
}

pub fn select_many(
    record_table: &str,
    columns: &Vec<&str>,
    filter: &str,
    order: &HashMap<&str, &str>,
    with_query_limit: &bool,
) -> String {
    let mut query = format!(
        "SELECT {} FROM \"hyperbase\".\"{}\"",
        columns.iter().map(|col| format!("\"{col}\"")).join(", "),
        record_table,
    );
    if filter.len() > 0 {
        query += &format!(" WHERE {filter}")
    }
    if order.len() > 0 {
        query += " ORDER BY";
        for (field, kind) in order {
            query += &format!(" \"{field}\" {kind}");
        }
    }
    if *with_query_limit {
        query += " LIMIT ?"
    }
    query + " ALLOW FILTERING"
}

pub fn count(record_table: &str, filter: &str) -> String {
    let mut query = format!("SELECT COUNT(1) FROM \"hyperbase\".\"{}\"", record_table);
    if filter.len() > 0 {
        query += &format!(" WHERE {filter}")
    }
    query + " ALLOW FILTERING"
}
