use anyhow::{Error, Result};
use chrono::{Duration, Utc};
use scylla::{
    frame::value::CqlTimestamp, serialize::value::SerializeCql, transport::session::TypedRowIter,
    CachingSession,
};
use uuid::Uuid;

use crate::{db::ScyllaDb, model::file::FileModel};

const INSERT: &str = "INSERT INTO \"hyperbase\".\"files\" (\"id\", \"created_by\", \"created_at\", \"updated_at\", \"bucket_id\", \"file_name\", \"content_type\", \"size\", \"public\") VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT \"id\", \"created_by\", \"created_at\", \"updated_at\", \"bucket_id\", \"file_name\", \"content_type\", \"size\", \"public\" FROM \"hyperbase\".\"files\" WHERE \"id\" = ?";
const SELECT_MANY_BY_BUCKET_ID: &str = "SELECT \"id\", \"created_by\", \"created_at\", \"updated_at\", \"bucket_id\", \"file_name\", \"content_type\", \"size\", \"public\" FROM \"hyperbase\".\"files\" WHERE \"bucket_id\" = ?";
const COUNT_MANY_BY_BUCKET_ID: &str = "SELECT COUNT(1) FROM \"hyperbase\".\"files\" WHERE \"bucket_id\" = ?";
const SELECT_MANY_BY_CREATED_BY_AND_BUCKET_ID: &str = "SELECT \"id\", \"created_by\", \"created_at\", \"updated_at\", \"bucket_id\", \"file_name\", \"content_type\", \"size\", \"public\" FROM \"hyperbase\".\"files\" WHERE \"created_by\" = ? AND \"bucket_id\" = ? ALLOW FILTERING";
const COUNT_MANY_BY_CREATED_BY_AND_BUCKET_ID: &str = "SELECT COUNT(1) FROM \"hyperbase\".\"files\" WHERE \"created_by\" = ? AND \"bucket_id\" = ? ALLOW FILTERING";
const SELECT_MANY_EXPIRE: &str = "SELECT \"id\", \"created_by\", \"created_at\", \"updated_at\", \"bucket_id\", \"file_name\", \"content_type\", \"size\", \"public\" FROM \"hyperbase\".\"files\" WHERE \"updated_at\" < ? ALLOW FILTERING";
const UPDATE: &str = "UPDATE \"hyperbase\".\"files\" SET \"created_by\" = ?, \"updated_at\" = ?, \"file_name\" = ?, \"public\" = ? WHERE \"id\" = ?";
const DELETE: &str = "DELETE FROM \"hyperbase\".\"files\" WHERE \"id\" = ?";

pub async fn init(cached_session: &CachingSession) {
    hb_log::info(Some("🔧"), "ScyllaDB: Setting up files table");

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"files\" (\"id\" uuid, \"created_by\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"bucket_id\" uuid, \"file_name\" text, \"content_type\" text, \"size\" bigint, \"public\" boolean, PRIMARY KEY (\"id\"))", &[]).await.unwrap();
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
        .add_prepared_statement(&COUNT_MANY_BY_BUCKET_ID.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT_MANY_BY_CREATED_BY_AND_BUCKET_ID.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&COUNT_MANY_BY_CREATED_BY_AND_BUCKET_ID.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT_MANY_EXPIRE.into())
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
        before_id: &Option<Uuid>,
        limit: &Option<i32>,
    ) -> Result<TypedRowIter<FileModel>> {
        let mut query = SELECT_MANY_BY_BUCKET_ID.to_owned();
        let mut values: Vec<Box<dyn SerializeCql + Send + Sync>> = Vec::new();
        values.push(Box::new(*bucket_id));
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

    pub async fn count_many_files_by_bucket_id(&self, bucket_id: &Uuid) -> Result<i64> {
        Ok(self
            .execute(COUNT_MANY_BY_BUCKET_ID, [bucket_id].as_ref())
            .await?
            .first_row_typed::<(i64,)>()?
            .0)
    }

    pub async fn select_many_files_by_created_by_and_bucket_id(
        &self,
        created_by: &Uuid,
        bucket_id: &Uuid,
        before_id: &Option<Uuid>,
        limit: &Option<i32>,
    ) -> Result<TypedRowIter<FileModel>> {
        let mut query = SELECT_MANY_BY_CREATED_BY_AND_BUCKET_ID.to_owned();
        let mut values: Vec<Box<dyn SerializeCql + Send + Sync>> = Vec::new();
        values.push(Box::new(*created_by));
        values.push(Box::new(*bucket_id));
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

    pub async fn count_many_files_by_created_by_and_bucket_id(
        &self,
        created_by: &Uuid,
        bucket_id: &Uuid,
    ) -> Result<i64> {
        Ok(self
            .execute(
                COUNT_MANY_BY_CREATED_BY_AND_BUCKET_ID,
                [created_by, bucket_id].as_ref(),
            )
            .await?
            .first_row_typed::<(i64,)>()?
            .0)
    }

    pub async fn select_many_expired_file(
        &self,
        ttl_seconds: &i64,
    ) -> Result<TypedRowIter<FileModel>> {
        Ok(self
            .execute(
                SELECT_MANY_EXPIRE,
                [CqlTimestamp(
                    Utc::now()
                        .checked_sub_signed(
                            Duration::try_seconds(*ttl_seconds)
                                .ok_or_else(|| Error::msg("bucket ttl is out of range."))?,
                        )
                        .ok_or_else(|| Error::msg("bucket ttl is out of range."))?
                        .timestamp_millis(),
                )]
                .as_ref(),
            )
            .await?
            .rows_typed()?)
    }

    pub async fn update_file(&self, value: &FileModel) -> Result<()> {
        self.execute(
            UPDATE,
            &(
                value.created_by(),
                value.updated_at(),
                value.file_name(),
                value.public(),
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
