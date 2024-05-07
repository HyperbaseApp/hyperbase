use std::net::SocketAddr;

use ahash::{HashSet, HashSetExt};
use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_dao::{
    admin::AdminDao,
    bucket::BucketDao,
    bucket_rule::BucketRuleDao,
    change::{ChangeDao, ChangeState, ChangeTable},
    collection::CollectionDao,
    collection_rule::CollectionRuleDao,
    file::FileDao,
    project::ProjectDao,
    record::RecordDao,
    token::TokenDao,
    Db,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

type ContentChannel = (SocketAddr, Uuid, Uuid, ContentMessage);
pub type ContentChannelSender = mpsc::UnboundedSender<ContentChannel>;
pub type ContentChannelReceiver = mpsc::UnboundedReceiver<ContentChannel>;

#[derive(Deserialize, Serialize)]
pub enum ContentMessage {
    Request {
        change_ids: Vec<Uuid>,
    },
    Response {
        changes_data: Vec<ContentChangeModel>,
    },
    Broadcast {
        change_data: ContentChangeModel,
    },
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ContentChangeModel {
    table: String,
    id: Uuid,
    state: String,
    updated_at: DateTime<Utc>,
    change_id: Uuid,
    data: Option<Vec<u8>>,
}

impl ContentChangeModel {
    pub async fn from_change_dao(db: &Db, change_data: &ChangeDao) -> Result<Self> {
        let data = if *change_data.state() == ChangeState::Upsert {
            Some(match change_data.table() {
                ChangeTable::Admin => {
                    let admin_data = AdminDao::db_select(&db, change_data.id()).await?;
                    admin_data.to_vec()?
                }
                ChangeTable::Project => {
                    let project_data = ProjectDao::db_select(&db, change_data.id()).await?;
                    project_data.to_vec()?
                }
                ChangeTable::Collection => {
                    let collection_data = CollectionDao::db_select(&db, change_data.id()).await?;
                    collection_data.to_vec()?
                }
                ChangeTable::Record(collection_id) => {
                    let collection_data = CollectionDao::db_select(&db, collection_id).await?;
                    let record_data = RecordDao::db_select(
                        &db,
                        change_data.id(),
                        &None,
                        &HashSet::new(),
                        &collection_data,
                        &true,
                    )
                    .await?;
                    record_data.to_vec()?
                }
                ChangeTable::Bucket => {
                    let bucket_data = BucketDao::db_select(&db, change_data.id()).await?;
                    bucket_data.to_vec()?
                }
                ChangeTable::File(bucket_id) => {
                    let bucket_data = BucketDao::db_select(&db, &bucket_id).await?;
                    let mut file_data =
                        FileDao::db_select(&db, &bucket_data, change_data.id()).await?;
                    file_data.populate_bytes(bucket_data.path()).await?;
                    file_data.to_vec()?
                }
                ChangeTable::Token => {
                    let token_data = TokenDao::db_select(&db, change_data.id()).await?;
                    token_data.to_vec()?
                }
                ChangeTable::CollectionRule => {
                    let collection_rule_data =
                        CollectionRuleDao::db_select(&db, change_data.id()).await?;
                    collection_rule_data.to_vec()?
                }
                ChangeTable::BucketRule => {
                    let bucket_rule_data = BucketRuleDao::db_select(&db, change_data.id()).await?;
                    bucket_rule_data.to_vec()?
                }
            })
        } else {
            None
        };

        Ok(Self {
            table: change_data.table().to_string(),
            id: *change_data.id(),
            state: change_data.state().to_str().to_owned(),
            updated_at: *change_data.updated_at(),
            change_id: *change_data.change_id(),
            data,
        })
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    pub fn change_id(&self) -> &Uuid {
        &self.change_id
    }

    pub async fn handle(&self, db: &Db) -> Result<()> {
        let change_data_table = ChangeTable::from_str(&self.table)?;
        let change_data_state = ChangeState::from_str(&self.state)?;
        let change_data = ChangeDao::raw_new(
            &change_data_table,
            &self.id,
            &change_data_state,
            &self.updated_at,
            &self.change_id,
        );
        change_data.db_upsert(db).await?;
        if let Some(data) = &self.data {
            match change_data_table {
                ChangeTable::Admin => {
                    let admin_data = AdminDao::from_bytes(data)?;
                    admin_data.db_upsert(db).await?;
                }
                ChangeTable::Project => {
                    let project_data = ProjectDao::from_bytes(data)?;
                    project_data.db_upsert(db).await?;
                }
                ChangeTable::Collection => {
                    let collection_data = CollectionDao::from_bytes(data)?;
                    collection_data.db_insert(db).await?;
                }
                ChangeTable::Record(_) => {
                    let record_data = RecordDao::from_bytes(data)?;
                    record_data.db_insert(db).await?;
                }
                ChangeTable::Bucket => {
                    let bucket_data = BucketDao::from_bytes(data)?;
                    bucket_data.db_insert(db).await?;
                }
                ChangeTable::File(_) => {
                    let file_data = FileDao::from_bytes(data)?;
                    let bucket_data = BucketDao::db_select(db, file_data.bucket_id()).await?;
                    file_data.save_from_bytes(db, bucket_data.path()).await?;
                }
                ChangeTable::Token => {
                    let token_data = TokenDao::from_bytes(data)?;
                    token_data.db_insert(db).await?;
                }
                ChangeTable::CollectionRule => {
                    let collection_rule_data = CollectionRuleDao::from_bytes(data)?;
                    collection_rule_data.db_insert(db).await?;
                }
                ChangeTable::BucketRule => {
                    let bucket_rule_data = BucketRuleDao::from_bytes(data)?;
                    bucket_rule_data.db_insert(db).await?;
                }
            }
        }
        Ok(())
    }
}
