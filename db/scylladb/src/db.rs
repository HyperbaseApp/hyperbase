use hb_config::DbScyllaConfig;
use scylla::{
    frame::value::ValueList, prepared_statement::PreparedStatement, query::Query,
    transport::errors::QueryError, QueryResult, Session, SessionBuilder,
};

use crate::prepared_statement::{
    admin::AdminPreparedStatement, collection::CollectionPreparedStatement,
    project::ProjectPreparedStatement, token::TokenPreparedStatement,
};

pub struct ScyllaDb {
    session: Session,
    prepared_statement: ScyllaPreparedStatement,
}

impl ScyllaDb {
    pub async fn new(config: &DbScyllaConfig) -> Self {
        let uri = format!("{}:{}", config.host(), config.port());
        let session = SessionBuilder::new().known_node(uri).build().await.unwrap();

        ScyllaDb::init(
            &session,
            ScyllaDbOpts {
                replication_factor: *config.replication_factor(),
            },
        )
        .await;

        ScyllaDb {
            prepared_statement: ScyllaPreparedStatement {
                admin: AdminPreparedStatement::new(&session).await,
                token: TokenPreparedStatement::new(&session).await,
                project: ProjectPreparedStatement::new(&session).await,
                collection: CollectionPreparedStatement::new(&session).await,
            },
            session,
        }
    }

    async fn init(session: &Session, opts: ScyllaDbOpts) {
        // Create keyspace
        session.query(format!("CREATE KEYSPACE IF NOT EXISTS ks WITH REPLICATION = {{'class' : 'NetworkTopologyStrategy', 'replication_factor' :{}}}", opts.replication_factor), &[]).await.unwrap();

        // Create types
        session
            .query(
                "CREATE TYPE IF NOT EXISTS ks.schema_field (\"name\" text, \"kind\" text, \"required\" boolean)",
                &[],
            )
            .await
            .unwrap();

        // Create tables
        session.query(format!("CREATE TABLE IF NOT EXISTS {} (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"email\" text, \"password_hash\" text, PRIMARY KEY (\"id\"))", AdminPreparedStatement::table_name()),&[]).await.unwrap();
        session.query(format!("CREATE TABLE IF NOT EXISTS {} (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"admin_id\" uuid, \"token\" text, \"expired_at\" timestamp, PRIMARY KEY (\"id\"))", TokenPreparedStatement::table_name()), &[]).await.unwrap();
        session.query(format!("CREATE TABLE IF NOT EXISTS {} (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"admin_id\" uuid, \"name\" text, PRIMARY KEY (\"id\"))", ProjectPreparedStatement::table_name()), &[]).await.unwrap();
        session.query(format!("CREATE TABLE IF NOT EXISTS {} (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"project_id\" uuid, \"name\" text, \"schema_fields\" list<frozen<schema_field>>, \"indexes\" list<text>, PRIMARY KEY (\"id\"))", CollectionPreparedStatement::table_name()), &[]).await.unwrap();
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
}

pub struct ScyllaDbOpts {
    pub replication_factor: i64,
}
