use scylla::CachingSession;

pub const INSERT: &str = "INSERT INTO \"hyperbase\".\"admins\" (\"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\") VALUES (?, ?, ?, ?, ?)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\" FROM \"hyperbase\".\"admins\" WHERE \"id\" = ?";
pub const SELECT_BY_EMAIL: &str= "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\" FROM \"hyperbase\".\"admins\" WHERE \"email\" = ?";
pub const UPDATE: &str = "UPDATE \"hyperbase\".\"admins\" SET \"updated_at\" = ?, \"email\" = ?, \"password_hash\" = ? WHERE \"id\" = ?";
pub const DELETE: &str = "DELETE FROM \"hyperbase\".\"admins\" WHERE \"id\" = ?";

pub async fn init(cached_session: &CachingSession) {
    hb_log::info(Some("🔧"), "ScyllaDB: Setting up admins table");

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"admins\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"email\" text, \"password_hash\" text, PRIMARY KEY (\"id\"))",&[]).await.unwrap();
    cached_session
        .get_session()
        .query(
            "CREATE INDEX IF NOT EXISTS ON \"hyperbase\".\"admins\" (\"email\")",
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
        .add_prepared_statement(&SELECT_BY_EMAIL.into())
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
