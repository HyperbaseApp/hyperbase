use scylla::{
    frame::value::ValueList,
    transport::{errors::QueryError, iterator::RowIterator},
    Bytes, CachingSession, QueryResult, SessionBuilder,
};

use crate::query::{
    admin, admin_password_reset, collection, keyspace, project, registration, token,
};

pub struct ScyllaDb {
    cached_session: CachingSession,
}

impl ScyllaDb {
    pub async fn new(
        host: &str,
        port: &str,
        replication_factor: &i64,
        cache_size: &usize,
        table_registration_ttl: &i64,
        table_reset_password_ttl: &i64,
    ) -> Self {
        hb_log::info(Some("⚡"), "ScyllaDb: Initializing component");

        let uri = format!("{}:{}", host, port);
        let cached_session: CachingSession = CachingSession::from(
            SessionBuilder::new().known_node(uri).build().await.unwrap(),
            *cache_size,
        );

        ScyllaDb::init(
            &cached_session,
            replication_factor,
            table_registration_ttl,
            table_reset_password_ttl,
        )
        .await;

        ScyllaDb { cached_session }
    }

    async fn init(
        cached_session: &CachingSession,
        replication_factor: &i64,
        table_registration_ttl: &i64,
        table_reset_password_ttl: &i64,
    ) {
        // Create keyspace
        keyspace::init(cached_session, replication_factor).await;

        // Create tables
        admin::init(cached_session).await;
        token::init(cached_session).await;
        project::init(cached_session).await;
        collection::init(cached_session).await;
        registration::init(cached_session, table_registration_ttl).await;
        admin_password_reset::init(cached_session, table_reset_password_ttl).await;
    }

    pub async fn session_query(
        &self,
        query: &str,
        values: impl ValueList,
    ) -> Result<QueryResult, QueryError> {
        self.cached_session.get_session().query(query, values).await
    }

    pub async fn execute(
        &self,
        query: &str,
        values: impl ValueList,
    ) -> Result<QueryResult, QueryError> {
        self.cached_session.execute(query, values).await
    }

    pub async fn execute_iter(
        &self,
        query: &str,
        values: impl ValueList,
    ) -> Result<RowIterator, QueryError> {
        self.cached_session.execute_iter(query, values).await
    }

    pub async fn execute_paged(
        &self,
        query: &str,
        values: impl ValueList,
        paging_state: Option<Bytes>,
    ) -> Result<QueryResult, QueryError> {
        self.cached_session
            .execute_paged(query, values, paging_state)
            .await
    }
}
