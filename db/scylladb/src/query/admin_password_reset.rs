use scylla::CachingSession;

pub const INSERT: &str = "INSERT INTO \"hyperbase\".\"admin_password_resets\" (\"id\", \"created_at\", \"updated_at\", \"admin_id\", \"code\") VALUES (?, ?, ?, ?, ?)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"code\" FROM \"hyperbase\".\"admin_password_resets\" WHERE \"id\" = ?";
pub const UPDATE: &str =
    "UPDATE \"hyperbase\".\"admin_password_resets\" SET \"updated_at\" = ?, \"code\" = ? WHERE \"id\" = ?";
pub const DELETE: &str = "DELETE FROM \"hyperbase\".\"admin_password_resets\" WHERE \"id\" = ?";

pub async fn init(cached_session: &CachingSession, ttl: &i64) {
    hb_log::info(
        Some("🔧"),
        "ScyllaDb: Setting up admin_password_resets table",
    );

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"admin_password_resets\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"admin_id\" uuid, \"code\" text, PRIMARY KEY (\"id\")) WITH default_time_to_live = ".to_owned() + &ttl.to_string(), &[]).await.unwrap();

    cached_session
        .add_prepared_statement(&INSERT.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT.into())
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
