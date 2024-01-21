use sqlx::{
    mysql::{MySqlArguments, MySqlPoolOptions, MySqlQueryResult, MySqlRow},
    query::{Query, QueryAs},
    Error, MySql, Pool,
};

use crate::query::{admin, admin_password_reset, bucket, collection, project, registration, token};

pub struct MysqlDb {
    pool: Pool<MySql>,
    table_registration_ttl: i64,
    table_reset_password_ttl: i64,
}

impl MysqlDb {
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
        hb_log::info(Some("⚡"), "MySQL: Initializing component");

        let url = format!("mysql://{user}:{password}@{host}:{port}/{db_name}");
        let pool = MySqlPoolOptions::new()
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
        query: Query<'_, MySql, MySqlArguments>,
    ) -> Result<MySqlQueryResult, Error> {
        query.persistent(false).execute(&self.pool).await
    }

    pub async fn execute(
        &self,
        query: Query<'_, MySql, MySqlArguments>,
    ) -> Result<MySqlQueryResult, Error> {
        query.execute(&self.pool).await
    }

    pub async fn fetch_one_unprepared<T: Send + Unpin + for<'r> sqlx::FromRow<'r, MySqlRow>>(
        &self,
        query: QueryAs<'_, MySql, T, MySqlArguments>,
    ) -> Result<T, Error> {
        Ok(query.persistent(false).fetch_one(&self.pool).await?)
    }

    pub async fn fetch_one<T: Send + Unpin + for<'r> sqlx::FromRow<'r, MySqlRow>>(
        &self,
        query: QueryAs<'_, MySql, T, MySqlArguments>,
    ) -> Result<T, Error> {
        Ok(query.fetch_one(&self.pool).await?)
    }

    pub async fn fetch_one_row(
        &self,
        query: Query<'_, MySql, MySqlArguments>,
    ) -> Result<MySqlRow, Error> {
        Ok(query.fetch_one(&self.pool).await?)
    }

    pub async fn fetch_all<T: Send + Unpin + for<'r> sqlx::FromRow<'r, MySqlRow>>(
        &self,
        query: QueryAs<'_, MySql, T, MySqlArguments>,
    ) -> Result<Vec<T>, Error> {
        query.fetch_all(&self.pool).await
    }

    pub async fn fetch_all_rows(
        &self,
        query: Query<'_, MySql, MySqlArguments>,
    ) -> Result<Vec<MySqlRow>, Error> {
        query.fetch_all(&self.pool).await
    }

    pub fn table_registration_ttl(&self) -> &i64 {
        &self.table_registration_ttl
    }

    pub fn table_reset_password_ttl(&self) -> &i64 {
        &self.table_reset_password_ttl
    }

    async fn init(pool: &Pool<MySql>) {
        admin::init(pool).await;
        token::init(pool).await;
        project::init(pool).await;
        collection::init(pool).await;
        bucket::init(pool).await;
        registration::init(pool).await;
        admin_password_reset::init(pool).await;
    }
}
