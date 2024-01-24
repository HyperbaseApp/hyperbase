use anyhow::Result;
use scylla::{transport::session::TypedRowIter, CachingSession};
use uuid::Uuid;

use crate::{db::ScyllaDb, model::file::FileModel};

const INSERT: &str = "INSERT INTO \"hyperbase\".\"files\" (\"id\", \"created_at\", \"updated_at\", \"bucket_id\", \"file_name\", \"content_type\", \"size\") VALUES (?, ?, ?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"bucket_id\", \"file_name\", \"content_type\", \"size\" FROM \"hyperbase\".\"files\" WHERE \"id\" = ?";
const SELECT_MANY_BY_BUCKET_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"bucket_id\", \"file_name\", \"content_type\", \"size\" FROM \"hyperbase\".\"files\" WHERE \"bucket_id\" = ?";
const UPDATE: &str = "UPDATE \"hyperbase\".\"files\" SET \"updated_at\" = ?, \"file_name\" = ? WHERE \"id\" = ?";
const DELETE: &str = "DELETE FROM \"hyperbase\".\"files\" WHERE \"id\" = ?";

pub async fn init(cached_session: &CachingSession) {
    hb_log::info(Some("🔧"), "ScyllaDB: Setting up files table");

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"files\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"bucket_id\" uuid, \"file_name\" text, \"content_type\" text, \"size\" bigint, PRIMARY KEY (\"id\"))", &[]).await.unwrap();
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
    ) -> Result<TypedRowIter<FileModel>> {
        Ok(self
            .execute(SELECT_MANY_BY_BUCKET_ID, [bucket_id].as_ref())
            .await?
            .rows_typed()?)
    }

    pub async fn update_file(&self, value: &FileModel) -> Result<()> {
        self.execute(UPDATE, &(value.updated_at(), value.file_name(), value.id()))
            .await?;
        Ok(())
    }

    pub async fn delete_file(&self, id: &Uuid) -> Result<()> {
        self.execute(DELETE, [id].as_ref()).await?;
        Ok(())
    }
}
