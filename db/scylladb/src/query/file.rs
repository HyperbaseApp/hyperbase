use anyhow::Result;
use scylla::{serialize::value::SerializeCql, transport::session::TypedRowIter, CachingSession};
use uuid::Uuid;

use crate::{db::ScyllaDb, model::file::FileModel};

const INSERT: &str = "INSERT INTO \"hyperbase\".\"files\" (\"id\", \"created_by\", \"created_at\", \"updated_at\", \"bucket_id\", \"file_name\", \"content_type\", \"size\") VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT \"id\", \"created_by\", \"created_at\", \"updated_at\", \"bucket_id\", \"file_name\", \"content_type\", \"size\" FROM \"hyperbase\".\"files\" WHERE \"id\" = ?";
const SELECT_MANY_BY_BUCKET_ID: &str = "SELECT \"id\", \"created_by\", \"created_at\", \"updated_at\", \"bucket_id\", \"file_name\", \"content_type\", \"size\" FROM \"hyperbase\".\"files\" WHERE \"bucket_id\" = ?";
const COUNT_MANY_BY_BUCKET_ID: &str =
    "SELECT COUNT(1) FROM \"hyperbase\".\"files\" WHERE \"bucket_id\" = ?";
const SELECT_MANY_BY_CREATED_BY_AND_BUCKET_ID: &str = "SELECT \"id\", \"created_by\", \"created_at\", \"updated_at\", \"bucket_id\", \"file_name\", \"content_type\", \"size\" FROM \"hyperbase\".\"files\" WHERE \"created_by\" = ? AND \"bucket_id\" = ?";
const COUNT_MANY_BY_CREATED_BY_AND_BUCKET_ID: &str =
    "SELECT COUNT(1) FROM \"hyperbase\".\"files\" WHERE \"created_by\" = ? AND \"bucket_id\" = ?";
const UPDATE: &str = "UPDATE \"hyperbase\".\"files\" SET \"created_by\" = ?, \"updated_at\" = ?, \"file_name\" = ? WHERE \"id\" = ?";
const DELETE: &str = "DELETE FROM \"hyperbase\".\"files\" WHERE \"id\" = ?";

pub async fn init(cached_session: &CachingSession) {
    hb_log::info(Some("🔧"), "ScyllaDB: Setting up files table");

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"files\" (\"id\" uuid, \"created_by\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"bucket_id\" uuid, \"file_name\" text, \"content_type\" text, \"size\" bigint, PRIMARY KEY (\"id\"))", &[]).await.unwrap();
    cached_session
        .get_session()
        .query(
            "CREATE INDEX IF NOT EXISTS ON \"hyperbase\".\"files\" (\"bucket_id\")",
            &[],
        )
        .await
        .unwrap();

    cached_session
        .add_prepared_statement(&INSERT.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT_MANY_BY_BUCKET_ID.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&UPDATE.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&DELETE.into())
        .await
        .unwrap();
}

impl ScyllaDb {
    pub async fn insert_file(&self, value: &FileModel) -> Result<()> {
        self.execute(INSERT, value).await?;
        Ok(())
    }

    pub async fn select_file(&self, id: &Uuid) -> Result<FileModel> {
        Ok(self
            .execute(SELECT, [id].as_ref())
            .await?
            .first_row_typed()?)
    }

    pub async fn select_many_files_by_bucket_id(
        &self,
        bucket_id: &Uuid,
        after_id: &Option<Uuid>,
        limit: &Option<i32>,
    ) -> Result<TypedRowIter<FileModel>> {
        let mut query = SELECT_MANY_BY_BUCKET_ID.to_owned();
        let mut values: Vec<Box<dyn SerializeCql + Send + Sync>> = Vec::new();
        values.push(Box::new(*bucket_id));
        if let Some(after_id) = after_id {
            query += " AND \"id\" < ?";
            values.push(Box::new(*after_id));
        }
        query += " ORDER BY \"id\" DESC";
        if let Some(limit) = limit {
            query += " LIMIT ?";
            values.push(Box::new(*limit));
        }
        query += " ALLOW FILTERING";
        Ok(self.execute(&query, &values).await?.rows_typed()?)
    }

    pub async fn count_many_files_by_bucket_id(
        &self,
        bucket_id: &Uuid,
        after_id: &Option<Uuid>,
    ) -> Result<i64> {
        let mut query = COUNT_MANY_BY_BUCKET_ID.to_owned();
        let mut values: Vec<Box<dyn SerializeCql + Send + Sync>> = Vec::new();
        values.push(Box::new(*bucket_id));
        if let Some(after_id) = after_id {
            query += " AND \"id\" < ?";
            values.push(Box::new(*after_id));
        }
        Ok(self
            .execute(&query, &values)
            .await?
            .first_row_typed::<(i64,)>()?
            .0)
    }

    pub async fn select_many_files_by_created_by_and_bucket_id(
        &self,
        created_by: &Uuid,
        bucket_id: &Uuid,
        after_id: &Option<Uuid>,
        limit: &Option<i32>,
    ) -> Result<TypedRowIter<FileModel>> {
        let mut query = SELECT_MANY_BY_CREATED_BY_AND_BUCKET_ID.to_owned();
        let mut values: Vec<Box<dyn SerializeCql + Send + Sync>> = Vec::new();
        values.push(Box::new(*created_by));
        values.push(Box::new(*bucket_id));
        if let Some(after_id) = after_id {
            query += " AND \"id\" < ?";
            values.push(Box::new(*after_id));
        }
        query += " ORDER BY \"id\" DESC";
        if let Some(limit) = limit {
            query += " LIMIT ?";
            values.push(Box::new(*limit));
        }
        query += " ALLOW FILTERING";
        Ok(self.execute(&query, &values).await?.rows_typed()?)
    }

    pub async fn count_many_files_by_created_by_and_bucket_id(
        &self,
        created_by: &Uuid,
        bucket_id: &Uuid,
        after_id: &Option<Uuid>,
    ) -> Result<i64> {
        let mut query = COUNT_MANY_BY_CREATED_BY_AND_BUCKET_ID.to_owned();
        let mut values: Vec<Box<dyn SerializeCql + Send + Sync>> = Vec::new();
        values.push(Box::new(*created_by));
        values.push(Box::new(*bucket_id));
        if let Some(after_id) = after_id {
            query += " AND \"id\" < ?";
            values.push(Box::new(*after_id));
        }
        Ok(self
            .execute(&query, &values)
            .await?
            .first_row_typed::<(i64,)>()?
            .0)
    }

    pub async fn update_file(&self, value: &FileModel) -> Result<()> {
        self.execute(
            UPDATE,
            &(
                value.created_by(),
                value.updated_at(),
                value.file_name(),
                value.id(),
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_file(&self, id: &Uuid) -> Result<()> {
        self.execute(DELETE, [id].as_ref()).await?;
        Ok(())
    }
}
