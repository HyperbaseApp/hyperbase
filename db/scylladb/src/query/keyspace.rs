use scylla::CachingSession;

pub async fn init(cached_session: &CachingSession, replication_factor: &i64) {
    hb_log::info(Some("🔧"), "ScyllaDb: Setting up hyperbase keyspace");

    cached_session.get_session().query("CREATE KEYSPACE IF NOT EXISTS \"hyperbase\" WITH REPLICATION = {'class' : 'NetworkTopologyStrategy', 'replication_factor' : ".to_owned() + &replication_factor.to_string() + "}", &[]).await.unwrap();
}
