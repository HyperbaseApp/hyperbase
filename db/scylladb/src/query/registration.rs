use scylla::CachingSession;

pub const INSERT: &str = "INSERT INTO \"hyperbase\".\"registrations\" (\"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\", \"code\") VALUES (?, ?, ?, ?, ?, ?)";
pub const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\", \"code\" FROM \"hyperbase\".\"registrations\" WHERE \"id\" = ?";
pub const UPDATE: &str = "UPDATE \"hyperbase\".\"registrations\" SET \"updated_at\" = ?, \"code\" = ? WHERE \"id\" = ?";
pub const DELETE: &str = "DELETE FROM \"hyperbase\".\"registrations\" WHERE \"id\" = ?";

pub async fn init(cached_session: &CachingSession, ttl: &u32) {
    hb_log::info(Some("🔧"), "ScyllaDB: Setting up registrations table");

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"registrations\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"email\" text, \"password_hash\" text, \"code\" text, PRIMARY KEY (\"id\")) WITH default_time_to_live = ".to_owned() + &ttl.to_string(), &[]).await.unwrap();

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
