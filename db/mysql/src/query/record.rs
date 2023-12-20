use ahash::{HashMap, HashSet};
use itertools::Itertools;

use crate::model::collection::SchemaFieldPropsModel;

pub fn create_table(
    record_table: &str,
    columns: &HashMap<String, SchemaFieldPropsModel>,
) -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS `{}` (`_id` binary(16), {}, PRIMARY KEY (`_id`)) ",
        record_table,
        columns
            .iter()
            .map(|(col, col_props)| format!("`{}` {}", col, col_props.internal_kind().to_str()))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub fn drop_table(record_table: &str) -> String {
    format!("DROP TABLE IF EXISTS `{record_table}`")
}

pub fn add_columns(record_table: &str, columns: &HashMap<String, SchemaFieldPropsModel>) -> String {
    format!(
        "ALTER TABLE `{}` ADD ({})",
        record_table,
        columns
            .iter()
            .map(|(col, col_props)| format!("`{}` {}", col, col_props.internal_kind().to_str()))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub fn drop_columns(record_table: &str, column_names: &HashSet<String>) -> String {
    format!(
        "ALTER TABLE `{}` DROP ({})",
        record_table,
        &column_names.iter().map(|col| format!("`{col}`")).join(", ")
    )
}

pub fn change_columns_type(
    record_table: &str,
    columns: &HashMap<String, SchemaFieldPropsModel>,
) -> String {
    format!(
        "ALTER TABLE `{}` {}",
        record_table,
        columns
            .iter()
            .map(|(col, col_props)| format!(
                "ALTER `{}` TYPE {}",
                col,
                col_props.internal_kind().to_str()
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub fn count_index(record_table: &str, index: &str) -> String {
    format!("SELECT COUNT(1) FROM `information_schema`.`STATISTICS` WHERE `TABLE_NAME` = '{record_table}' and `INDEX_NAME` = '{index}'")
}

pub fn create_index(record_table: &str, index: &str) -> String {
    format!("CREATE INDEX IF NOT EXISTS '{record_table}_{index}' ON `{record_table}` ('{index}')")
}

pub fn drop_index(record_table: &str, index: &str) -> String {
    format!("DROP INDEX IF EXISTS `{record_table}_{index}`")
}

pub fn insert(record_table: &str, columns: &Vec<String>) -> String {
    let mut cols = "".to_owned();
    let mut vals = "".to_owned();
    for (idx, col) in columns.iter().enumerate() {
        cols += &format!("`{col}`");
        vals += &format!("${}", idx + 1);
        if idx < columns.len() - 1 {
            cols += ", ";
            vals += ", ";
        }
    }
    format!("INSERT INTO `{record_table}` ({cols}) VALUES ({vals})")
}

pub fn update(record_table: &str, columns: &Vec<String>) -> String {
    format!(
        "UPDATE `{}` SET {} WHERE `_id` = ?",
        record_table,
        columns.iter().map(|col| format!("`{col}` = ?")).join(", ")
    )
}

pub fn delete(record_table: &str, columns: &HashSet<String>) -> String {
    format!(
        "DELETE FROM `{}` WHERE {}",
        record_table,
        columns.iter().map(|col| format!("`{col}` = ?")).join(", ")
    )
}
