use scylla::{
    frame::value::ValueList, prepared_statement::PreparedStatement, query::Query,
    transport::errors::QueryError, QueryResult, Session, SessionBuilder,
};

use crate::prepared_statement::{
    admin::AdminPreparedStatement, admin_password_reset::AdminPasswordResetPreparedStatement,
    collection::CollectionPreparedStatement, project::ProjectPreparedStatement,
    registration::RegistrationPreparedStatement, token::TokenPreparedStatement,
};

pub struct ScyllaDb {
    session: Session,
    prepared_statement: ScyllaPreparedStatement,
}

impl ScyllaDb {
    pub async fn new(host: &str, port: &str, replication_factor: &i64, temp_ttl: &i64) -> Self {
        let uri = format!("{}:{}", host, port);
        let session = SessionBuilder::new().known_node(uri).build().await.unwrap();

        ScyllaDb::init(&session, replication_factor, temp_ttl).await;

        ScyllaDb {
            prepared_statement: ScyllaPreparedStatement {
                admin: AdminPreparedStatement::new(&session).await,
                token: TokenPreparedStatement::new(&session).await,
                project: ProjectPreparedStatement::new(&session).await,
                collection: CollectionPreparedStatement::new(&session).await,
                registration: RegistrationPreparedStatement::new(&session).await,
                admin_password_reset: AdminPasswordResetPreparedStatement::new(&session).await,
            },
            session,
        }
    }

    async fn init(session: &Session, replication_factor: &i64, temp_ttl: &i64) {
        // Create keyspace
        session.query(format!("CREATE KEYSPACE IF NOT EXISTS ks WITH REPLICATION = {{'class' : 'NetworkTopologyStrategy', 'replication_factor' :{}}}", replication_factor), &[]).await.unwrap();

        // Create types
        session
            .query(
                "CREATE TYPE IF NOT EXISTS ks.schema_field (\"name\" text, \"kind\" text, \"required\" boolean)",
                &[],
            )
            .await
            .unwrap();

        // Create tables
        // admins
        session.query(format!("CREATE TABLE IF NOT EXISTS {} (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"email\" text, \"password_hash\" text, PRIMARY KEY (\"id\"))", AdminPreparedStatement::table_name()),&[]).await.unwrap();
        session.query(format!("CREATE INDEX IF NOT EXISTS ON {}(\"email\")", AdminPreparedStatement::table_name()), &[]).await.unwrap();
        // tokens
        session.query(format!("CREATE TABLE IF NOT EXISTS {} (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"admin_id\" uuid, \"token\" text, \"expired_at\" timestamp, PRIMARY KEY (\"id\"))", TokenPreparedStatement::table_name()), &[]).await.unwrap();
        // projects
        session.query(format!("CREATE TABLE IF NOT EXISTS {} (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"admin_id\" uuid, \"name\" text, PRIMARY KEY (\"id\"))", ProjectPreparedStatement::table_name()), &[]).await.unwrap();
        // collections
        session.query(format!("CREATE TABLE IF NOT EXISTS {} (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"project_id\" uuid, \"name\" text, \"schema_fields\" list<frozen<schema_field>>, \"indexes\" list<text>, PRIMARY KEY (\"id\"))", CollectionPreparedStatement::table_name()), &[]).await.unwrap();
        // registrations
        session.query(format!("CREATE TABLE IF NOT EXISTS {} (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"email\" text, \"password_hash\" text, \"code\" text, PRIMARY KEY (\"id\")) WITH default_time_to_live = {}", RegistrationPreparedStatement::table_name(), temp_ttl ), &[]).await.unwrap();
        // admin_password_resets
        session.query(format!("CREATE TABLE IF NOT EXISTS {} (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"email\" text, \"code\" text, PRIMARY KEY (\"id\")) WITH default_time_to_live = {}", AdminPasswordResetPreparedStatement::table_name(), temp_ttl), &[]).await.unwrap();
    }

    pub fn prepared_statement(&self) -> &ScyllaPreparedStatement {
        &self.prepared_statement
    }

    pub async fn execute(
        &self,
        prepared: &PreparedStatement,
        values: impl ValueList,
    ) -> Result<QueryResult, QueryError> {
        self.session.execute(prepared, values).await
    }

    pub async fn query(
        &self,
        query: impl Into<Query>,
        values: impl ValueList,
    ) -> Result<QueryResult, QueryError> {
        self.session.query(query, values).await
    }
}

pub struct ScyllaPreparedStatement {
    admin: AdminPreparedStatement,
    token: TokenPreparedStatement,
    project: ProjectPreparedStatement,
    collection: CollectionPreparedStatement,

    registration: RegistrationPreparedStatement,
    admin_password_reset: AdminPasswordResetPreparedStatement,
}

impl ScyllaPreparedStatement {
    pub fn admin(&self) -> &AdminPreparedStatement {
        &self.admin
    }

    pub fn token(&self) -> &TokenPreparedStatement {
        &self.token
    }

    pub fn project(&self) -> &ProjectPreparedStatement {
        &self.project
    }

    pub fn collection(&self) -> &CollectionPreparedStatement {
        &self.collection
    }

    pub fn registration(&self) -> &RegistrationPreparedStatement {
        &self.registration
    }

    pub fn admin_password_reset(&self) -> &AdminPasswordResetPreparedStatement {
        &self.admin_password_reset
    }
}
