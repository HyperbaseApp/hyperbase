use anyhow::Result;
use scylla::{serialize::value::SerializeCql, transport::session::TypedRowIter, CachingSession};
use uuid::Uuid;

use crate::{db::ScyllaDb, model::log::LogModel};

const INSERT: &str = "INSERT INTO \"hyperbase\".\"logs\" (\"id\", \"created_at\", \"admin_id\", \"project_id\", \"kind\", \"message\") VALUES (?, ?, ?, ?, ?, ?)";
const SELECT_MANY_BY_ADMIN_ID_AND_PROJECT_ID: &str = "SELECT \"id\", \"created_at\", \"admin_id\", \"project_id\", \"kind\", \"message\" FROM \"hyperbase\".\"logs\" WHERE \"admin_id\" = ? AND \"project_id\" = ? ALLOW FILTERING";
const COUNT_MANY_BY_ADMIN_ID_AND_PROJECT_ID: &str = "SELECT COUNT(1) FROM \"hyperbase\".\"logs\" WHERE \"admin_id\" = ? AND \"project_id\" = ? ALLOW FILTERING";

pub async fn init(cached_session: &CachingSession, ttl: &u32) {
    hb_log::info(Some("🔧"), "ScyllaDB: logs up logs table");

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"logs\" (\"id\" uuid, \"created_at\" timestamp, \"admin_id\" uuid, \"project_id\" uuid, \"kind\" text, \"message\" text, PRIMARY KEY (\"id\")) WITH default_time_to_live = ".to_owned()+&ttl.to_string(), &[]).await.unwrap();

    cached_session
        .add_prepared_statement(&INSERT.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT_MANY_BY_ADMIN_ID_AND_PROJECT_ID.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&COUNT_MANY_BY_ADMIN_ID_AND_PROJECT_ID.into())
        .await
        .unwrap();
}

impl ScyllaDb {
    pub async fn insert_log(&self, value: &LogModel) -> Result<()> {
        self.execute(INSERT, value).await?;
        Ok(())
    }

    pub async fn select_many_logs_by_admin_id_and_project_id(
        &self,
        admin_id: &Uuid,
        project_id: &Uuid,
        before_id: &Option<Uuid>,
        limit: &Option<i32>,
    ) -> Result<TypedRowIter<LogModel>> {
        let mut query = SELECT_MANY_BY_ADMIN_ID_AND_PROJECT_ID.to_owned();
        let mut values: Vec<Box<dyn SerializeCql + Send + Sync>> = Vec::new();
        values.push(Box::new(*admin_id));
        values.push(Box::new(*project_id));
        if let Some(before_id) = before_id {
            query += " AND \"id\" < ?";
            values.push(Box::new(*before_id));
        }
        query += " ORDER BY \"id\" DESC";
        if let Some(limit) = limit {
            query += " LIMIT ?";
            values.push(Box::new(*limit));
        }
        query += " ALLOW FILTERING";
        Ok(self.execute(&query, &values).await?.rows_typed()?)
    }

    pub async fn count_many_logs_by_admin_id_and_project_id(
        &self,
        admin_id: &Uuid,
        project_id: &Uuid,
    ) -> Result<i64> {
        Ok(self
            .execute(
                COUNT_MANY_BY_ADMIN_ID_AND_PROJECT_ID,
                [admin_id, project_id].as_ref(),
            )
            .await?
            .first_row_typed::<(i64,)>()?
            .0)
    }
}
