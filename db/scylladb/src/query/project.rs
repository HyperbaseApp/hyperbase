use scylla::CachingSession;

pub const INSERT: &str = "INSERT INTO \"hyperbase\".\"projects\" (\"id\", \"created_at\", \"updated_at\", \"admin_id\", \"name\") VALUES (?, ?, ?, ?, ?)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"name\" FROM \"hyperbase\".\"projects\" WHERE \"id\" = ?";
pub const SELECT_MANY_BY_ADMIN_ID :  &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"name\" FROM \"hyperbase\".\"projects\" WHERE \"admin_id\" = ?";
pub const UPDATE: &str =
    "UPDATE \"hyperbase\".\"projects\" SET \"updated_at\" = ?, \"name\" = ? WHERE \"id\" = ?";
pub const DELETE: &str = "DELETE FROM \"hyperbase\".\"projects\" WHERE \"id\" = ?";

pub async fn init(cached_session: &CachingSession) {
    hb_log::info(Some("🔧"), "ScyllaDb: Setting up projects table");

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"projects\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"admin_id\" uuid, \"name\" text, PRIMARY KEY (\"id\"))", &[]).await.unwrap();
    cached_session
        .get_session()
        .query(
            "CREATE INDEX IF NOT EXISTS ON \"hyperbase\".\"projects\" (\"admin_id\")",
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
        .add_prepared_statement(&UPDATE.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&DELETE.into())
        .await
        .unwrap();
}
