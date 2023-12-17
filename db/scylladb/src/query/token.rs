use scylla::CachingSession;

pub const INSERT: &str = "INSERT INTO \"hyperbase\".\"tokens\" (\"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"rules\", \"expired_at\") VALUES (?, ?, ?, ?, ?, ?, ?)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"rules\", \"expired_at\" FROM \"hyperbase\".\"tokens\" WHERE \"id\" = ?";
pub const SELECT_MANY_BY_ADMIN_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"rules\", \"expired_at\" FROM \"hyperbase\".\"tokens\" WHERE \"admin_id\" = ?";
pub const SELECT_BY_TOKEN: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"rules\", \"expired_at\" FROM \"hyperbase\".\"tokens\" WHERE \"token\" = ?";
pub const UPDATE: &str = "UPDATE \"hyperbase\".\"tokens\" SET \"updated_at\" = ?, \"rules\" = ?, \"expired_at\" = ? WHERE \"id\" = ?";
pub const DELETE: &str = "DELETE FROM \"hyperbase\".\"tokens\" WHERE \"id\" = ?";

pub async fn init(cached_session: &CachingSession) {
    hb_log::info(Some("🔧"), "ScyllaDB: Setting up tokens table");

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"tokens\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"admin_id\" uuid, \"token\" text, \"rules\" map<uuid, tinyint>, \"expired_at\" timestamp, PRIMARY KEY (\"id\"))", &[]).await.unwrap();
    cached_session
        .get_session()
        .query(
            "CREATE INDEX IF NOT EXISTS ON \"hyperbase\".\"tokens\" (\"admin_id\")",
            &[],
        )
        .await
        .unwrap();
    cached_session
        .get_session()
        .query(
            "CREATE INDEX IF NOT EXISTS ON \"hyperbase\".\"tokens\" (\"token\")",
            &[],
        )
        .await
        .unwrap();

    cached_session
        .add_prepared_statement(&INSERT.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT_MANY_BY_ADMIN_ID.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT_BY_TOKEN.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&UPDATE.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&DELETE.into())
        .await
        .unwrap();
}
