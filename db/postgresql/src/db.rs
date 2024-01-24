use sqlx::{
    postgres::{PgArguments, PgPoolOptions, PgQueryResult, PgRow},
    query::{Query, QueryAs},
    Error, Pool, Postgres,
};

use crate::query::{
    admin, admin_password_reset, bucket, collection, file, project, registration, token,
};

pub struct PostgresDb {
    pool: Pool<Postgres>,
    table_registration_ttl: i64,
    table_reset_password_ttl: i64,
}

impl PostgresDb {
    pub async fn new(
        user: &str,
        password: &str,
        host: &str,
        port: &str,
        db_name: &str,
        max_connections: &u32,
        table_registration_ttl: &i64,
        table_reset_password_ttl: &i64,
    ) -> Self {
        hb_log::info(Some("⚡"), "PostgreSQL: Initializing component");

        let url = format!("postgres://{user}:{password}@{host}:{port}/{db_name}");
        let pool = PgPoolOptions::new()
            .max_connections(*max_connections)
            .connect(&url)
            .await
            .unwrap();

        Self::init(&pool).await;

        Self {
            pool,
            table_registration_ttl: *table_registration_ttl,
            table_reset_password_ttl: *table_reset_password_ttl,
        }
    }

    pub async fn execute_unprepared(
        &self,
        query: Query<'_, Postgres, PgArguments>,
    ) -> Result<PgQueryResult, Error> {
        query.persistent(false).execute(&self.pool).await
    }

    pub async fn execute(
        &self,
        query: Query<'_, Postgres, PgArguments>,
    ) -> Result<PgQueryResult, Error> {
        query.execute(&self.pool).await
    }

    pub async fn fetch_one_unprepared<T: Send + Unpin + for<'r> sqlx::FromRow<'r, PgRow>>(
        &self,
        query: QueryAs<'_, Postgres, T, PgArguments>,
    ) -> Result<T, Error> {
        Ok(query.persistent(false).fetch_one(&self.pool).await?)
    }

    pub async fn fetch_one<T: Send + Unpin + for<'r> sqlx::FromRow<'r, PgRow>>(
        &self,
        query: QueryAs<'_, Postgres, T, PgArguments>,
    ) -> Result<T, Error> {
        Ok(query.fetch_one(&self.pool).await?)
    }

    pub async fn fetch_one_row(
        &self,
        query: Query<'_, Postgres, PgArguments>,
    ) -> Result<PgRow, Error> {
        Ok(query.fetch_one(&self.pool).await?)
    }

    pub async fn fetch_all<T: Send + Unpin + for<'r> sqlx::FromRow<'r, PgRow>>(
        &self,
        query: QueryAs<'_, Postgres, T, PgArguments>,
    ) -> Result<Vec<T>, Error> {
        query.fetch_all(&self.pool).await
    }

    pub async fn fetch_all_rows(
        &self,
        query: Query<'_, Postgres, PgArguments>,
    ) -> Result<Vec<PgRow>, Error> {
        query.fetch_all(&self.pool).await
    }

    pub fn table_registration_ttl(&self) -> &i64 {
        &self.table_registration_ttl
    }

    pub fn table_reset_password_ttl(&self) -> &i64 {
        &self.table_reset_password_ttl
    }

    async fn init(pool: &Pool<Postgres>) {
        admin::init(pool).await;
        token::init(pool).await;
        project::init(pool).await;
        collection::init(pool).await;
        bucket::init(pool).await;
        file::init(pool).await;
        registration::init(pool).await;
        admin_password_reset::init(pool).await;
    }
}
