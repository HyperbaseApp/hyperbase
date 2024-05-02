use anyhow::Result;
use sqlx::{Executor, Pool, Postgres};
use uuid::Uuid;

use crate::{db::PostgresDb, model::admin::AdminModel};

const INSERT: &str = "INSERT INTO \"admins\" (\"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\") VALUES ($1, $2, $3, $4, $5)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\" FROM \"admins\" WHERE \"id\" = $1";
const SELECT_BY_EMAIL: &str= "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\" FROM \"admins\" WHERE \"email\" = $1";
const SELECT_ALL: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\" FROM \"admins\"";
const UPDATE: &str = "UPDATE \"admins\" SET \"updated_at\" = $1, \"email\" = $2, \"password_hash\" = $3 WHERE \"id\" = $4";
const DELETE: &str = "DELETE FROM \"admins\" WHERE \"id\" = $1";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "[PostgreSQL] Setting up admins table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"admins\" (\"id\" uuid, \"created_at\" timestamptz, \"updated_at\" timestamptz, \"email\" text, \"password_hash\" text, PRIMARY KEY (\"id\"))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(SELECT),
        pool.prepare(SELECT_BY_EMAIL),
        pool.prepare(UPDATE),
        pool.prepare(DELETE),
    )
    .unwrap();
}

impl PostgresDb {
    pub async fn insert_admin(&self, value: &AdminModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT)
                .bind(value.id())
                .bind(value.created_at())
                .bind(value.updated_at())
                .bind(value.email())
                .bind(value.password_hash()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_admin(&self, id: &Uuid) -> Result<AdminModel> {
        Ok(self.fetch_one(sqlx::query_as(SELECT).bind(id)).await?)
    }

    pub async fn select_admin_by_email(&self, email: &str) -> Result<AdminModel> {
        Ok(self
            .fetch_one(sqlx::query_as(SELECT_BY_EMAIL).bind(email))
            .await?)
    }

    pub async fn select_many_admins_after_id_with_limit(
        &self,
        after_id: &Option<Uuid>,
        limit: &i32,
    ) -> Result<Vec<AdminModel>> {
        let mut query = SELECT_ALL.to_owned();
        let mut values_count = 0;

        if after_id.is_some() {
            values_count += 1;
            query += &format!(" WHERE \"id\" > ${values_count}");
        }

        values_count += 1;
        query += &format!(" LIMIT ${values_count} ORDER BY \"id\" ASC");

        let mut query = sqlx::query_as(&query);
        if let Some(after_id) = after_id {
            query = query.bind(after_id);
        }

        query = query.bind(limit);

        Ok(self.fetch_all(query).await?)
    }

    pub async fn update_admin(&self, value: &AdminModel) -> Result<()> {
        self.execute(
            sqlx::query(UPDATE)
                .bind(value.updated_at())
                .bind(value.email())
                .bind(value.password_hash())
                .bind(value.id()),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_admin(&self, id: &Uuid) -> Result<()> {
        self.execute(sqlx::query(DELETE).bind(id)).await?;
        Ok(())
    }
}
