use anyhow::Result;
use sqlx::{Executor, Pool, Postgres};
use uuid::Uuid;

use crate::{db::PostgresDb, model::token::TokenModel};

const INSERT: &str = "INSERT INTO \"tokens\" (\"id\", \"created_at\", \"updated_at\", \"project_id\", \"admin_id\", \"name\", \"token\", \"allow_anonymous\", \"expired_at\") VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"admin_id\", \"name\", \"token\", \"allow_anonymous\", \"expired_at\" FROM \"tokens\" WHERE \"id\" = $1";
const SELECT_MANY_BY_ADMIN_ID_AND_PROJECT_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"admin_id\", \"name\", \"token\", \"allow_anonymous\", \"expired_at\" FROM \"tokens\" WHERE \"admin_id\" = $1 AND \"project_id\" = $2 ORDER BY \"id\" DESC";
const SELECT_MANY_BY_PROJECT_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"admin_id\", \"name\", \"token\", \"allow_anonymous\", \"expired_at\" FROM \"tokens\" WHERE \"project_id\" = $1 ORDER BY \"id\" DESC";
const SELECT_ALL: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"admin_id\", \"name\", \"token\", \"allow_anonymous\", \"expired_at\" FROM \"tokens\"";
const UPDATE: &str = "UPDATE \"tokens\" SET \"updated_at\" = $1, \"admin_id\" = $2, \"name\" = $3, \"allow_anonymous\" = $4, \"expired_at\" = $5 WHERE \"id\" = $6";
const DELETE: &str = "DELETE FROM \"tokens\" WHERE \"id\" = $1";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "[PostgreSQL] Setting up tokens table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"tokens\" (\"id\" uuid, \"created_at\" timestamptz, \"updated_at\" timestamptz, \"project_id\" uuid, \"admin_id\" uuid, \"name\" text, \"token\" text, \"allow_anonymous\" boolean, \"expired_at\" timestamptz, PRIMARY KEY (\"id\"))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(SELECT),
        pool.prepare(SELECT_MANY_BY_ADMIN_ID_AND_PROJECT_ID),
        pool.prepare(SELECT_MANY_BY_PROJECT_ID),
        pool.prepare(UPDATE),
        pool.prepare(DELETE),
    )
    .unwrap();
}

impl PostgresDb {
    pub async fn insert_token(&self, value: &TokenModel) -> Result<()> {
        self.execute(
            sqlx::query(INSERT)
                .bind(value.id())
                .bind(value.created_at())
                .bind(value.updated_at())
                .bind(value.project_id())
                .bind(value.admin_id())
                .bind(value.name())
                .bind(value.token())
                .bind(value.allow_anonymous())
                .bind(value.expired_at()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_token(&self, id: &Uuid) -> Result<TokenModel> {
        Ok(self.fetch_one(sqlx::query_as(SELECT).bind(id)).await?)
    }

    pub async fn select_many_tokens_by_admin_id_and_project_id(
        &self,
        admin_id: &Uuid,
        project_id: &Uuid,
    ) -> Result<Vec<TokenModel>> {
        Ok(self
            .fetch_all(
                sqlx::query_as(SELECT_MANY_BY_ADMIN_ID_AND_PROJECT_ID)
                    .bind(admin_id)
                    .bind(project_id),
            )
            .await?)
    }

    pub async fn select_many_tokens_by_project_id(
        &self,
        project_id: &Uuid,
    ) -> Result<Vec<TokenModel>> {
        Ok(self
            .fetch_all(sqlx::query_as(SELECT_MANY_BY_PROJECT_ID).bind(project_id))
            .await?)
    }

    pub async fn select_many_tokens_after_id_with_limit(
        &self,
        after_id: &Option<Uuid>,
        limit: &i32,
    ) -> Result<Vec<TokenModel>> {
        let mut query = SELECT_ALL.to_owned();
        let mut values_count = 0;

        if after_id.is_some() {
            values_count += 1;
            query += &format!(" WHERE \"id\" > ${values_count}");
        }

        values_count += 1;
        query += &format!(" ORDER BY \"id\" ASC LIMIT ${values_count}");

        let mut query = sqlx::query_as(&query);
        if let Some(after_id) = after_id {
            query = query.bind(after_id);
        }

        query = query.bind(limit);

        Ok(self.fetch_all(query).await?)
    }

    pub async fn update_token(&self, value: &TokenModel) -> Result<()> {
        self.execute(
            sqlx::query(UPDATE)
                .bind(value.updated_at())
                .bind(value.admin_id())
                .bind(value.name())
                .bind(value.allow_anonymous())
                .bind(value.expired_at())
                .bind(value.id()),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_token(&self, id: &Uuid) -> Result<()> {
        self.execute(sqlx::query(DELETE).bind(id)).await?;
        Ok(())
    }
}
