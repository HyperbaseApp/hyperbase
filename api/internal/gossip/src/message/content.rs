use std::net::SocketAddr;

use ahash::{HashSet, HashSetExt};
use anyhow::{Error, Result};
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
    table: ContentChangeTable,
    id: Uuid,
    state: ContentChangeState,
    timestamp: DateTime<Utc>,
    change_id: Uuid,
    data: Option<Vec<u8>>,
}

impl ContentChangeModel {
    pub async fn from_change_dao(db: &Db, change_data: &ChangeDao) -> Result<Self> {
        let data = match change_data.state() {
            ChangeState::Insert => match change_data.table() {
                ChangeTable::Collection => {
                    let collection_data = CollectionDao::db_select(db, change_data.id()).await?;
                    Some(collection_data.to_vec()?)
                }
                table => {
                    return Err(Error::msg(format!(
                        "Invalid combination of change data state 'insert' and table '{}'",
                        table.to_string()
                    )));
                }
            },
            ChangeState::Upsert => match change_data.table() {
                ChangeTable::Admin => {
                    let admin_data = AdminDao::db_select(db, change_data.id()).await?;
                    Some(admin_data.to_vec()?)
                }
                ChangeTable::Project => {
                    let project_data = ProjectDao::db_select(db, change_data.id()).await?;
                    Some(project_data.to_vec()?)
                }
                ChangeTable::Collection => {
                    return Err(Error::msg(
                        "Invalid combination of change data state 'upsert' and table 'collection'",
                    ));
                }
                ChangeTable::Record(collection_id) => {
                    let collection_data = CollectionDao::db_select(db, collection_id).await?;
                    let record_data = RecordDao::db_select(
                        db,
                        change_data.id(),
                        &None,
                        &HashSet::new(),
                        &collection_data,
                        &true,
                    )
                    .await?;
                    Some(record_data.to_vec()?)
                }
                ChangeTable::Bucket => {
                    let bucket_data = BucketDao::db_select(db, change_data.id()).await?;
                    Some(bucket_data.to_vec()?)
                }
                ChangeTable::File(bucket_id) => {
                    let bucket_data = BucketDao::db_select(db, bucket_id).await?;
                    let mut file_data =
                        FileDao::db_select(db, &bucket_data, change_data.id()).await?;
                    file_data.populate_file_bytes(bucket_data.path()).await?;
                    Some(file_data.to_vec()?)
                }
                ChangeTable::Token => {
                    let token_data = TokenDao::db_select(db, change_data.id()).await?;
                    Some(token_data.to_vec()?)
                }
                ChangeTable::CollectionRule => {
                    let collection_rule_data =
                        CollectionRuleDao::db_select(db, change_data.id()).await?;
                    Some(collection_rule_data.to_vec()?)
                }
                ChangeTable::BucketRule => {
                    let bucket_rule_data = BucketRuleDao::db_select(db, change_data.id()).await?;
                    Some(bucket_rule_data.to_vec()?)
                }
            },
            ChangeState::Update => match change_data.table() {
                ChangeTable::Collection => {
                    let collection_data = CollectionDao::db_select(db, change_data.id()).await?;
                    Some(collection_data.to_vec()?)
                }
                table => {
                    return Err(Error::msg(format!(
                        "Invalid combination of change data state 'update' and table '{}'",
                        table.to_string()
                    )));
                }
            },
            ChangeState::Delete => match change_data.table() {
                ChangeTable::Record(collection_id) => {
                    let collection_data = CollectionDao::db_select(db, collection_id).await?;
                    let record_data = RecordDao::db_select(
                        db,
                        change_data.id(),
                        &None,
                        &HashSet::new(),
                        &collection_data,
                        &true,
                    )
                    .await?;
                    Some(record_data.to_vec()?)
                }
                ChangeTable::File(bucket_id) => {
                    let bucket_data = BucketDao::db_select(db, bucket_id).await?;
                    let mut file_data =
                        FileDao::db_select(db, &bucket_data, change_data.id()).await?;
                    file_data.populate_file_bytes(bucket_data.path()).await?;
                    Some(file_data.to_vec()?)
                }
                _ => None,
            },
        };

        Ok(Self {
            table: ContentChangeTable::from_dao(change_data.table()),
            id: *change_data.id(),
            state: ContentChangeState::from_dao(change_data.state()),
            timestamp: *change_data.timestamp(),
            change_id: *change_data.change_id(),
            data,
        })
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    pub fn change_id(&self) -> &Uuid {
        &self.change_id
    }

    pub async fn handle(&self, db: &Db) -> Result<()> {
        let change_data_table = self.table.to_dao();
        let change_data_state = self.state.to_dao();

        match change_data_state {
            ChangeState::Insert => {
                if let Some(data) = &self.data {
                    match change_data_table {
                        ChangeTable::Collection => {
                            let collection_data = CollectionDao::from_bytes(data)?;
                            collection_data.db_insert(db).await?;
                        }
                        table => {
                            return Err(Error::msg(format!(
                                "Invalid combination of change data state 'update' and table '{}'",
                                table.to_string()
                            )));
                        }
                    }
                } else {
                    return Err(Error::msg("Change state must contains data"));
                }
            }
            ChangeState::Upsert => {
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
                            return Err(Error::msg(
                                "Invalid combination of change data state 'upsert' and table 'collection'",
                            ));
                        }
                        ChangeTable::Record(_) => {
                            let record_data = RecordDao::from_bytes(data)?;
                            if let Some(id) = record_data.id() {
                                let _ = RecordDao::db_delete(
                                    db,
                                    record_data.collection_id(),
                                    id,
                                    &None,
                                )
                                .await;
                            } else {
                                return Err(Error::msg("Record data must contains _id field"));
                            }
                            record_data.db_insert(db).await?;
                        }
                        ChangeTable::Bucket => {
                            let bucket_data = BucketDao::from_bytes(data)?;
                            bucket_data.db_upsert(db).await?;
                        }
                        ChangeTable::File(_) => {
                            let file_data = FileDao::from_bytes(data)?;
                            let bucket_data =
                                BucketDao::db_select(db, file_data.bucket_id()).await?;
                            let _ = FileDao::delete(db, &bucket_data, file_data.id()).await;
                            file_data.save_from_bytes(db, bucket_data.path()).await?;
                        }
                        ChangeTable::Token => {
                            let token_data = TokenDao::from_bytes(data)?;
                            token_data.db_upsert(db).await?;
                        }
                        ChangeTable::CollectionRule => {
                            let collection_rule_data = CollectionRuleDao::from_bytes(data)?;
                            collection_rule_data.db_upsert(db).await?;
                        }
                        ChangeTable::BucketRule => {
                            let bucket_rule_data = BucketRuleDao::from_bytes(data)?;
                            bucket_rule_data.db_upsert(db).await?;
                        }
                    }
                } else {
                    return Err(Error::msg("Change state must contains data"));
                }
            }
            ChangeState::Update => {
                if let Some(data) = &self.data {
                    match change_data_table {
                        ChangeTable::Collection => {
                            let mut collection_data = CollectionDao::from_bytes(data)?;
                            collection_data.db_update_raw(db).await?;
                        }
                        table => {
                            return Err(Error::msg(format!(
                                "Invalid combination of change data state 'update' and table '{}'",
                                table.to_string()
                            )));
                        }
                    }
                } else {
                    return Err(Error::msg("Change state must contains data"));
                }
            }
            ChangeState::Delete => match change_data_table {
                ChangeTable::Admin => {
                    AdminDao::db_delete(db, &self.id).await?;
                }
                ChangeTable::Project => {
                    ProjectDao::db_delete(db, &self.id).await?;
                }
                ChangeTable::Collection => {
                    CollectionDao::db_delete(db, &self.id).await?;
                }
                ChangeTable::Record(_) => {
                    if let Some(data) = &self.data {
                        let record_data = RecordDao::from_bytes(data)?;
                        if let Some(record_id) = record_data.id() {
                            let collection_data =
                                CollectionDao::db_select(db, record_data.collection_id()).await?;
                            RecordDao::db_delete(db, collection_data.id(), record_id, &None)
                                .await?;
                        }
                    } else {
                        return Err(Error::msg("Change state must contains data"));
                    }
                }
                ChangeTable::Bucket => {
                    BucketDao::db_delete(db, &self.id).await?;
                }
                ChangeTable::File(_) => {
                    if let Some(data) = &self.data {
                        let file_data = FileDao::from_bytes(data)?;
                        let bucket_data = BucketDao::db_select(db, file_data.bucket_id()).await?;
                        FileDao::delete(db, &bucket_data, file_data.id()).await?;
                    } else {
                        return Err(Error::msg("Change state must contains data"));
                    }
                }
                ChangeTable::Token => {
                    TokenDao::db_delete(db, &self.id).await?;
                }
                ChangeTable::CollectionRule => {
                    CollectionRuleDao::db_delete(db, &self.id).await?;
                }
                ChangeTable::BucketRule => {
                    BucketRuleDao::db_delete(db, &self.id).await?;
                }
            },
        }

        let change_data = ChangeDao::raw_new(
            &change_data_table,
            &self.id,
            &change_data_state,
            &self.timestamp,
            &self.change_id,
        );
        change_data.db_upsert(db).await?;

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Clone, Copy)]
enum ContentChangeTable {
    Admin,
    Project,
    Collection,
    Record(Uuid),
    Bucket,
    File(Uuid),
    Token,
    CollectionRule,
    BucketRule,
}

impl ContentChangeTable {
    fn from_dao(change_table: &ChangeTable) -> Self {
        match change_table {
            ChangeTable::Admin => Self::Admin,
            ChangeTable::Project => Self::Project,
            ChangeTable::Collection => Self::Collection,
            ChangeTable::Record(collection_id) => Self::Record(*collection_id),
            ChangeTable::Bucket => Self::Bucket,
            ChangeTable::File(bucket_id) => Self::File(*bucket_id),
            ChangeTable::Token => Self::Token,
            ChangeTable::CollectionRule => Self::CollectionRule,
            ChangeTable::BucketRule => Self::BucketRule,
        }
    }

    fn to_dao(&self) -> ChangeTable {
        match self {
            Self::Admin => ChangeTable::Admin,
            Self::Project => ChangeTable::Project,
            Self::Collection => ChangeTable::Collection,
            Self::Record(collection_id) => ChangeTable::Record(*collection_id),
            Self::Bucket => ChangeTable::Bucket,
            Self::File(bucket_id) => ChangeTable::File(*bucket_id),
            Self::Token => ChangeTable::Token,
            Self::CollectionRule => ChangeTable::CollectionRule,
            Self::BucketRule => ChangeTable::BucketRule,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy)]
enum ContentChangeState {
    Insert,
    Upsert,
    Update,
    Delete,
}

impl ContentChangeState {
    fn from_dao(change_state: &ChangeState) -> Self {
        match change_state {
            ChangeState::Insert => Self::Insert,
            ChangeState::Upsert => Self::Upsert,
            ChangeState::Update => Self::Update,
            ChangeState::Delete => Self::Delete,
        }
    }

    fn to_dao(&self) -> ChangeState {
        match self {
            Self::Insert => ChangeState::Insert,
            Self::Upsert => ChangeState::Upsert,
            Self::Update => ChangeState::Update,
            Self::Delete => ChangeState::Delete,
        }
    }
}
