use anyhow::Result;
use scylla::{transport::session::TypedRowIter, CachingSession};
use uuid::Uuid;

use crate::{db::ScyllaDb, model::collection_rule::CollectionRuleModel};

const INSERT: &str = "INSERT INTO \"hyperbase\".\"collection_rules\" (\"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"collection_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\") VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
const SELECT: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"collection_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\" FROM \"hyperbase\".\"collection_rules\" WHERE \"id\" = ?";
const SELECT_BY_TOKEN_ID_AND_COLLECTION_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"collection_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\" FROM \"hyperbase\".\"collection_rules\" WHERE \"token_id\" = ? AND \"collection_id\" = ? ALLOW FILTERING";
const SELECT_MANY_BY_TOKEN_ID: &str = "SELECT \"id\", \"created_at\", \"updated_at\", \"project_id\", \"token_id\", \"collection_id\", \"find_one\", \"find_many\", \"insert_one\", \"update_one\", \"delete_one\" FROM \"hyperbase\".\"collection_rules\" WHERE \"token_id\" = ?";
const UPDATE: &str = "UPDATE \"hyperbase\".\"collection_rules\" SET \"updated_at\" = ?, \"find_one\" = ?, \"find_many\" = ?, \"insert_one\" = ?, \"update_one\" = ?, \"delete_one\" = ? WHERE \"id\" = ?";
const DELETE: &str = "DELETE FROM \"hyperbase\".\"collection_rules\" WHERE \"id\" = ?";

pub async fn init(cached_session: &CachingSession) {
    hb_log::info(Some("🔧"), "ScyllaDB: Setting up collection_rules table");

    cached_session.get_session().query("CREATE TABLE IF NOT EXISTS \"hyperbase\".\"collection_rules\" (\"id\" uuid, \"created_at\" timestamp, \"updated_at\" timestamp, \"project_id\" uuid, \"token_id\" uuid, \"collection_id\" uuid, \"find_one\" text, \"find_many\" text, \"insert_one\" boolean, \"update_one\" text, \"delete_one\" text, PRIMARY KEY (\"id\"))", &[]).await.unwrap();
    cached_session
        .get_session()
        .query(
            "CREATE INDEX IF NOT EXISTS ON \"hyperbase\".\"collection_rules\" (\"token_id\")",
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
        .add_prepared_statement(&SELECT_BY_TOKEN_ID_AND_COLLECTION_ID.into())
        .await
        .unwrap();
    cached_session
        .add_prepared_statement(&SELECT_MANY_BY_TOKEN_ID.into())
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
    pub async fn insert_collection_rule(&self, value: &CollectionRuleModel) -> Result<()> {
        self.execute(INSERT, value).await?;
        Ok(())
    }

    pub async fn select_collection_rule(&self, id: &Uuid) -> Result<CollectionRuleModel> {
        Ok(self
            .execute(SELECT, [id].as_ref())
            .await?
            .first_row_typed()?)
    }

    pub async fn select_collection_rule_by_token_id_and_collection_id(
        &self,
        token_id: &Uuid,
        collection_id: &Uuid,
    ) -> Result<CollectionRuleModel> {
        Ok(self
            .execute(SELECT, [token_id, collection_id].as_ref())
            .await?
            .first_row_typed()?)
    }

    pub async fn select_many_collection_rules_by_token_id(
        &self,
        token_id: &Uuid,
    ) -> Result<TypedRowIter<CollectionRuleModel>> {
        Ok(self
            .execute(SELECT_MANY_BY_TOKEN_ID, [token_id].as_ref())
            .await?
            .rows_typed()?)
    }

    pub async fn update_collection_rule(&self, value: &CollectionRuleModel) -> Result<()> {
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

    pub async fn delete_collection_rule(&self, id: &Uuid) -> Result<()> {
        self.execute(DELETE, [id].as_ref()).await?;
        Ok(())
    }

    pub async fn delete_many_collection_rules_by_token_id(&self, token_id: &Uuid) -> Result<()> {
        let collection_rules_data = self
            .select_many_collection_rules_by_token_id(token_id)
            .await?;
        let mut deletes = Vec::new();
        for collection_rule_data in collection_rules_data {
            deletes.push(self.execute(DELETE, (*collection_rule_data?.id(),)));
        }
        futures::future::join_all(deletes).await;
        Ok(())
    }
}
