use anyhow::Result;
use scylla::{serialize::value::SerializeCql, transport::session::TypedRowIter, CachingSession};
use uuid::Uuid;

use crate::{db::ScyllaDb, model::bucket_rule::BucketRuleModel};

const INSERT: &str = "INSERT INTO \"hyperbase\".\"bucket_rules\" (\"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"bucket_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\") VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"bucket_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\" FROM \"hyperbase\".\"bucket_rules\" WHERE \"id\" = ?";
const SELECT_BY_TOKEN_ID_AND_BUCKET_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"bucket_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\" FROM \"hyperbase\".\"bucket_rules\" WHERE \"token_id\" = ? AND \"bucket_id\" = ? ALLOW FILTERING";
const SELECT_MANY_BY_TOKEN_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"bucket_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\" FROM \"hyperbase\".\"bucket_rules\" WHERE \"token_id\" = ?";
const SELECT_MANY_BY_BUCKET_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"bucket_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\" FROM \"hyperbase\".\"bucket_rules\" WHERE \"bucket_id\" = ?";
const SELECT_ALL: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"bucket_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\" FROM \"hyperbase\".\"bucket_rules\"";
const UPDATE: &str = "UPDATE \"hyperbase\".\"bucket_rules\" SET \"updated_at\" = ?, \"find_one\" = ?, \"find_many\" = ?, \"insert_one\" = ?, \"update_one\" = ?, \"delete_one\" = ? WHERE \"id\" = ?";
const DELETE: &str = "DELETE FROM \"hyperbase\".\"bucket_rules\" WHERE \"id\" = ?";

pub async fn init(cached_session: &CachingSession) {
    hb_log::info(Some("🔧"), "[ScyllaDB] Setting up bucket_rules table");

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"bucket_rules\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"project_id\" uuid, \"token_id\" uuid, \"bucket_id\" uuid, \"find_one\" text, \"find_many\" text, \"insert_one\" boolean, \"update_one\" text, \"delete_one\" text, PRIMARY KEY (\"id\"))", &[]).await.unwrap();
    cached_session
        .get_session()
        .query(
            "CREATE INDEX IF NOT EXISTS ON \"hyperbase\".\"bucket_rules\" (\"token_id\")",
            &[],
        )
        .await
        .unwrap();
    cached_session
        .get_session()
        .query(
            "CREATE INDEX IF NOT EXISTS ON \"hyperbase\".\"bucket_rules\" (\"bucket_id\")",
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
        .add_prepared_statement(&SELECT_BY_TOKEN_ID_AND_BUCKET_ID.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT_MANY_BY_TOKEN_ID.into())
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
    pub async fn insert_bucket_rule(&self, value: &BucketRuleModel) -> Result<()> {
        self.execute(INSERT, value).await?;
        Ok(())
    }

    pub async fn select_bucket_rule(&self, id: &Uuid) -> Result<BucketRuleModel> {
        Ok(self
            .execute(SELECT, [id].as_ref())
            .await?
            .first_row_typed()?)
    }

    pub async fn select_bucket_rule_by_token_id_and_bucket_id(
        &self,
        token_id: &Uuid,
        bucket_id: &Uuid,
    ) -> Result<BucketRuleModel> {
        Ok(self
            .execute(
                SELECT_BY_TOKEN_ID_AND_BUCKET_ID,
                [token_id, bucket_id].as_ref(),
            )
            .await?
            .first_row_typed()?)
    }

    pub async fn select_many_bucket_rules_by_token_id(
        &self,
        token_id: &Uuid,
    ) -> Result<TypedRowIter<BucketRuleModel>> {
        Ok(self
            .execute(SELECT_MANY_BY_TOKEN_ID, [token_id].as_ref())
            .await?
            .rows_typed()?)
    }

    pub async fn select_many_bucket_rules_by_bucket_id(
        &self,
        bucket_id: &Uuid,
    ) -> Result<TypedRowIter<BucketRuleModel>> {
        Ok(self
            .execute(SELECT_MANY_BY_BUCKET_ID, [bucket_id].as_ref())
            .await?
            .rows_typed()?)
    }

    pub async fn select_many_bucket_rules_after_id_with_limit(
        &self,
        after_id: &Option<Uuid>,
        limit: &i32,
    ) -> Result<TypedRowIter<BucketRuleModel>> {
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

    pub async fn update_bucket_rule(&self, value: &BucketRuleModel) -> Result<()> {
        self.execute(
            UPDATE,
            &(
                value.updated_at(),
                value.find_one(),
                value.find_many(),
                value.insert_one(),
                value.update_one(),
                value.delete_one(),
                value.id(),
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn delete_bucket_rule(&self, id: &Uuid) -> Result<()> {
        self.execute(DELETE, [id].as_ref()).await?;
        Ok(())
    }

    pub async fn delete_many_bucket_rules_by_token_id(&self, token_id: &Uuid) -> Result<()> {
        let bucket_rules_data = self.select_many_bucket_rules_by_token_id(token_id).await?;
        let mut deletes = Vec::new();
        for bucket_rule_data in bucket_rules_data {
            deletes.push(self.execute(DELETE, (*bucket_rule_data?.id(),)));
        }
        futures::future::join_all(deletes).await;
        Ok(())
    }

    pub async fn delete_many_bucket_rules_by_bucket_id(&self, bucket_id: &Uuid) -> Result<()> {
        let bucket_rules_data = self
            .select_many_bucket_rules_by_bucket_id(bucket_id)
            .await?;
        let mut deletes = Vec::new();
        for bucket_rule_data in bucket_rules_data {
            deletes.push(self.execute(DELETE, (*bucket_rule_data?.id(),)));
        }
        futures::future::join_all(deletes).await;
        Ok(())
    }
}
