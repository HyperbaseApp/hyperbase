use sqlx::{
    query::{Query, QueryAs},
    sqlite::{SqliteArguments, SqlitePoolOptions, SqliteQueryResult, SqliteRow},
    Error, Pool, Sqlite,
};

use crate::query::{
    admin, admin_password_reset, bucket, collection, file, project, registration, token,
};

pub struct SqliteDb {
    pool: Pool<Sqlite>,
    table_registration_ttl: i64,
    table_reset_password_ttl: i64,
}

impl SqliteDb {
    pub async fn new(
        path: &str,
        max_connections: &u32,
        table_registration_ttl: &i64,
        table_reset_password_ttl: &i64,
    ) -> Self {
        hb_log::info(Some("⚡"), "SQLite: Initializing component");

        let url = format!("sqlite:{path}?mode=rwc");
        let pool = SqlitePoolOptions::new()
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

    pub async fn execute_unprepared<'a>(
        &self,
        query: Query<'a, Sqlite, SqliteArguments<'a>>,
    ) -> Result<SqliteQueryResult, Error> {
        query.persistent(false).execute(&self.pool).await
    }

    pub async fn execute<'a>(
        &self,
        query: Query<'a, Sqlite, SqliteArguments<'a>>,
    ) -> Result<SqliteQueryResult, Error> {
        query.execute(&self.pool).await
    }

    pub async fn fetch_one_unprepared<
        'a,
        T: Send + Unpin + for<'r> sqlx::FromRow<'r, SqliteRow>,
    >(
        &self,
        query: QueryAs<'a, Sqlite, T, SqliteArguments<'a>>,
    ) -> Result<T, Error> {
        Ok(query.persistent(false).fetch_one(&self.pool).await?)
    }

    pub async fn fetch_one<'a, T: Send + Unpin + for<'r> sqlx::FromRow<'r, SqliteRow>>(
        &self,
        query: QueryAs<'a, Sqlite, T, SqliteArguments<'a>>,
    ) -> Result<T, Error> {
        Ok(query.fetch_one(&self.pool).await?)
    }

    pub async fn fetch_one_row<'a>(
        &self,
        query: Query<'a, Sqlite, SqliteArguments<'a>>,
    ) -> Result<SqliteRow, Error> {
        Ok(query.fetch_one(&self.pool).await?)
    }

    pub async fn fetch_all<'a, T: Send + Unpin + for<'r> sqlx::FromRow<'r, SqliteRow>>(
        &self,
        query: QueryAs<'a, Sqlite, T, SqliteArguments<'a>>,
    ) -> Result<Vec<T>, Error> {
        query.fetch_all(&self.pool).await
    }

    pub async fn fetch_all_rows<'a>(
        &self,
        query: Query<'a, Sqlite, SqliteArguments<'a>>,
    ) -> Result<Vec<SqliteRow>, Error> {
        query.fetch_all(&self.pool).await
    }

    pub fn table_registration_ttl(&self) -> &i64 {
        &self.table_registration_ttl
    }

    pub fn table_reset_password_ttl(&self) -> &i64 {
        &self.table_reset_password_ttl
    }

    async fn init(pool: &Pool<Sqlite>) {
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
