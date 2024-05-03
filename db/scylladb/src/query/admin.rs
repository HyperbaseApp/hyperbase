use anyhow::Result;
use scylla::{serialize::value::SerializeCql, transport::session::TypedRowIter, CachingSession};
use uuid::Uuid;

use crate::{db::ScyllaDb, model::admin::AdminModel};

const INSERT: &str = "INSERT INTO \"hyperbase\".\"admins\" (\"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\") VALUES (?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\" FROM \"hyperbase\".\"admins\" WHERE \"id\" = ?";
const SELECT_BY_EMAIL: &str= "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\" FROM \"hyperbase\".\"admins\" WHERE \"email\" = ?";
const SELECT_ALL: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\" FROM \"hyperbase\".\"admins\"";
const UPDATE: &str = "UPDATE \"hyperbase\".\"admins\" SET \"updated_at\" = ?, \"email\" = ?, \"password_hash\" = ? WHERE \"id\" = ?";
const DELETE: &str = "DELETE FROM \"hyperbase\".\"admins\" WHERE \"id\" = ?";

pub async fn init(cached_session: &CachingSession) {
    hb_log::info(Some("🔧"), "[ScyllaDB] Setting up admins table");

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"admins\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"email\" text, \"password_hash\" text, PRIMARY KEY (\"id\"))",&[]).await.unwrap();
    cached_session
        .get_session()
        .query(
            "CREATE INDEX IF NOT EXISTS ON \"hyperbase\".\"admins\" (\"email\")",
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
        .add_prepared_statement(&SELECT_BY_EMAIL.into())
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
    pub async fn insert_admin(&self, value: &AdminModel) -> Result<()> {
        self.execute(INSERT, value).await?;
        Ok(())
    }

    pub async fn select_admin(&self, id: &Uuid) -> Result<AdminModel> {
        Ok(self
            .execute(SELECT, [id].as_ref())
            .await?
            .first_row_typed()?)
    }

    pub async fn select_admin_by_email(&self, email: &str) -> Result<AdminModel> {
        Ok(self
            .execute(SELECT_BY_EMAIL, [email].as_ref())
            .await?
            .first_row_typed()?)
    }

    pub async fn select_many_admins_after_id_with_limit(
        &self,
        after_id: &Option<Uuid>,
        limit: &i32,
    ) -> Result<TypedRowIter<AdminModel>> {
        let mut query = SELECT_ALL.to_owned();
        let mut values: Vec<Box<dyn SerializeCql>> = Vec::with_capacity(2);

        if let Some(after_id) = after_id {
            values.push(Box::new(after_id));
            query += " WHERE \"id\" > ?";
        }

        values.push(Box::new(limit));
        query += " ORDER BY \"id\" ASC LIMIT ?";

        Ok(self.execute(&query, values).await?.rows_typed()?)
    }

    pub async fn update_admin(&self, value: &AdminModel) -> Result<()> {
        self.execute(
            UPDATE,
            &(
                value.updated_at(),
                value.email(),
                value.password_hash(),
                value.id(),
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_admin(&self, id: &Uuid) -> Result<()> {
        self.execute(DELETE, [id].as_ref()).await?;
        Ok(())
    }
}
