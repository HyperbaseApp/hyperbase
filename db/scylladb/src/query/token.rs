use scylla::CachingSession;

pub const INSERT: &str = "INSERT INTO \"ks\".\"tokens\" (\"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"expired_at\") VALUES (?, ?, ?, ?, ?, ?)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"expired_at\" FROM \"ks\".\"tokens\" WHERE \"id\" = ?";
pub const SELECT_MANY_BY_ADMIN_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"expired_at\" FROM \"ks\".\"tokens\" WHERE \"admin_id\" = ?";
pub const SELECT_BY_TOKEN: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"expired_at\" FROM \"ks\".\"tokens\" WHERE \"token\" = ?";
pub const UPDATE: &str =
    "UPDATE \"ks\".\"tokens\" SET \"updated_at\" = ?, \"expired_at\" = ? WHERE \"id\" = ?";
pub const DELETE: &str = "DELETE FROM \"ks\".\"tokens\" WHERE \"id\" = ?";

pub async fn init(cached_session: &CachingSession) {
    hb_log::info(Some("🔧"), "ScyllaDb: Setting up tokens table");

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"ks\".\"tokens\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"admin_id\" uuid, \"token\" text, \"expired_at\" timestamp, PRIMARY KEY (\"id\"))", &[]).await.unwrap();
    cached_session
        .get_session()
        .query(
            "CREATE INDEX IF NOT EXISTS ON \"ks\".\"tokens\" (\"admin_id\")",
            &[],
        )
        .await
        .unwrap();
    cached_session
        .get_session()
        .query(
            "CREATE INDEX IF NOT EXISTS ON \"ks\".\"tokens\" (\"token\")",
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
