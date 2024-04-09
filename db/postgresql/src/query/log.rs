use anyhow::{Error, Result};
use chrono::{Duration, Utc};
use sqlx::{Executor, Pool, Postgres};
use uuid::Uuid;

use crate::{db::PostgresDb, model::log::LogModel};

const INSERT: &str = "INSERT INTO \"logs\" (\"id\", \"created_at\", \"admin_id\", \"project_id\", \"kind\", \"message\") VALUES ($1, $2, $3, $4, $5, $6)";
const SELECT_MANY_BY_ADMIN_ID_AND_PROJECT_ID: &str = "SELECT \"id\", \"created_at\", \"admin_id\", \"project_id\", \"kind\", \"message\" FROM \"logs\" WHERE \"admin_id\" = $1 AND \"project_id\" = $2";
const COUNT_MANY_BY_ADMIN_ID_AND_PROJECT_ID: &str = "SELECT COUNT(1) FROM \"logs\" WHERE \"admin_id\" = $1 AND \"project_id\" = $2";
const DELETE_EXPIRE: &str = "DELETE FROM \"logs\" WHERE \"created_at\" < $1";

pub async fn init(pool: &Pool<Postgres>) {
    hb_log::info(Some("🔧"), "PostgreSQL: Setting up logs table");

    pool.execute("CREATE TABLE IF NOT EXISTS \"logs\" (\"id\" uuid, \"created_at\" timestamptz, \"admin_id\" uuid, \"project_id\" uuid, \"kind\" text, \"message\" text, PRIMARY KEY (\"id\"))").await.unwrap();

    tokio::try_join!(
        pool.prepare(INSERT),
        pool.prepare(SELECT_MANY_BY_ADMIN_ID_AND_PROJECT_ID),
        pool.prepare(COUNT_MANY_BY_ADMIN_ID_AND_PROJECT_ID),
        pool.prepare(DELETE_EXPIRE)
    )
    .unwrap();
}

impl PostgresDb {
    pub async fn insert_log(&self, value: &LogModel) -> Result<()> {
        let _ = self.delete_expired_logs().await?;

        self.execute(
            sqlx::query(INSERT)
                .bind(value.id())
                .bind(value.created_at())
                .bind(value.admin_id())
                .bind(value.project_id())
                .bind(value.kind())
                .bind(value.message()),
        )
        .await?;
        Ok(())
    }

    pub async fn select_many_logs_by_admin_id_and_project_id(
        &self,
        admin_id: &Uuid,
        project_id: &Uuid,
        before_id: &Option<Uuid>,
        limit: &Option<i32>,
    ) -> Result<Vec<LogModel>> {
        let _ = self.delete_expired_logs().await?;

        let mut sql = SELECT_MANY_BY_ADMIN_ID_AND_PROJECT_ID.to_owned();
        let mut count_values = 2;
        if before_id.is_some() {
            count_values += 1;
            sql += &format!(" AND \"id\" < ${count_values}");
        }
        sql += " ORDER BY \"id\" DESC";
        if limit.is_some() {
            count_values += 1;
            sql += &format!(" LIMIT ${count_values}");
        }

        let mut query = sqlx::query_as(&sql).bind(admin_id).bind(project_id);
        if let Some(before_id) = before_id {
            query = query.bind(before_id);
        }
        if let Some(limit) = limit {
            query = query.bind(limit);
        }

        Ok(self.fetch_all(query).await?)
    }

    pub async fn count_many_logs_by_admin_id_and_project_id(
        &self,
        admin_id: &Uuid,
        project_id: &Uuid,
    ) -> Result<i64> {
        let _ = self.delete_expired_logs().await?;

        Ok(self
            .fetch_one::<(i64,)>(
                sqlx::query_as(COUNT_MANY_BY_ADMIN_ID_AND_PROJECT_ID)
                    .bind(admin_id)
                    .bind(project_id),
            )
            .await?
            .0)
    }

    async fn delete_expired_logs(&self) -> Result<()> {
        self.execute(
            sqlx::query(DELETE_EXPIRE).bind(
                Utc::now()
                    .checked_sub_signed(
                        Duration::try_seconds(*self.table_log_ttl())
                            .ok_or_else(|| Error::msg("table_log_ttl is out of range."))?,
                    )
                    .ok_or_else(|| Error::msg("table_log_ttl is out of range."))?,
            ),
        )
        .await?;
        Ok(())
    }
}
