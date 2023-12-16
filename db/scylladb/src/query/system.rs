pub const COUNT_TABLE: &str = "SELECT COUNT(1) FROM \"system_schema\".\"tables\" WHERE \"keyspace_name\" = 'hyperbase' AND \"table_name\" = ?";
