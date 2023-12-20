pub const COUNT_TABLE: &str = "SELECT COUNT(1) FROM \"information_schema\".\"tables\" WHERE \"table_name\" = $1";
