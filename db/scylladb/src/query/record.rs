use ahash::{HashMap, HashSet};
use itertools::Itertools;

use crate::model::collection::SchemaFieldPropsModel;

pub fn create_table(
    record_table: &str,
    columns: &HashMap<String, SchemaFieldPropsModel>,
) -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS \"hyperbase\".\"{}\" (\"_collection_id\" uuid, \"_id\" uuid, \"_created_by\" uuid, \"_updated_at\" timestamp, {}PRIMARY KEY (\"_collection_id\", \"_id\")) WITH CLUSTERING ORDER BY (\"_id\" DESC)",
        record_table,
        columns
            .iter()
            .map(|(col, col_props)| format!("\"{}\" {}, ", col, col_props.internal_kind().to_str()))
            .join("")
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

pub fn add_columns(record_table: &str, columns: &HashMap<String, SchemaFieldPropsModel>) -> String {
    format!(
        "ALTER TABLE \"hyperbase\".\"{}\" ADD ({})",
        record_table,
        columns
            .iter()
            .map(|(col, col_props)| format!("\"{}\" {}", col, col_props.internal_kind().to_str()))
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
    columns: &HashMap<String, SchemaFieldPropsModel>,
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
            .join(", ")
    )
}

pub fn insert(record_table: &str, columns: &Vec<&str>) -> String {
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

pub fn select(record_table: &str, columns: &Vec<&str>) -> String {
    format!(
        "SELECT {} FROM \"hyperbase\".\"{}\" WHERE \"_collection_id\" = ? AND \"_id\" = ?",
        columns.iter().map(|col| format!("\"{col}\"")).join(", "),
        record_table
    )
}

pub fn select_by_id_and_created_by(record_table: &str, columns: &Vec<&str>) -> String {
    select(record_table, columns) + " AND \"_created_by\" = ?"
}

pub fn select_many(
    record_table: &str,
    columns: &Vec<&str>,
    filter: &str,
    groups: &Vec<&str>,
    with_query_limit: &bool,
) -> String {
    let mut query = format!(
        "SELECT {} FROM \"hyperbase\".\"{}\"",
        columns
            .iter()
            .map(|col| {
                if *col == "COUNT(1)" {
                    (*col).to_owned()
                } else {
                    format!("\"{col}\"")
                }
            })
            .join(", "),
        record_table,
    );
    if filter.len() > 0 {
        query += &format!(" WHERE {filter}")
    }
    if groups.len() > 0 {
        query += " GROUP BY";
        let mut count = 0;
        for group in groups {
            if count > 0 {
                query += ",";
            }
            query += &format!(" \"{group}\"");
            count += 1;
        }
    }
    if *with_query_limit {
        query += " LIMIT ?"
    }
    if filter.len() > 0 {
        query += " ALLOW FILTERING"
    }
    query
}

pub fn update(record_table: &str, columns: &Vec<&str>) -> String {
    format!(
        "UPDATE \"hyperbase\".\"{}\" SET {} WHERE \"_collection_id\" = ? AND \"_id\" = ?",
        record_table,
        columns
            .iter()
            .map(|col| format!("\"{col}\" = ?"))
            .join(", ")
    )
}

pub fn delete(record_table: &str, columns: &Vec<&str>) -> String {
    format!(
        "DELETE FROM \"hyperbase\".\"{}\" WHERE {}",
        record_table,
        columns
            .iter()
            .map(|col| format!("\"{col}\" = ?"))
            .join(" AND ")
    )
}

pub fn count(record_table: &str, filter: &str, groups: &Vec<&str>) -> String {
    let mut query = format!("SELECT COUNT(1) FROM \"hyperbase\".\"{}\"", record_table);
    if filter.len() > 0 {
        query += &format!(" WHERE {filter}")
    }
    if groups.len() > 0 {
        query += " GROUP BY";
        let mut count = 0;
        for group in groups {
            if count > 0 {
                query += ",";
            }
            query += &format!(" \"{group}\"");
            count += 1;
        }
    }
    if filter.len() > 0 {
        query += " ALLOW FILTERING";
    }
    query
}
