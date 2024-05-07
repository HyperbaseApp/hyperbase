use ahash::{HashSet, HashSetExt};
use anyhow::{Error, Result};
use chrono::DateTime;
use futures::future;
use uuid::Uuid;

use crate::{
    admin::AdminDao,
    bucket::BucketDao,
    bucket_rule::BucketRuleDao,
    change::{ChangeDao, ChangeState, ChangeTable},
    collection::CollectionDao,
    collection_rule::CollectionRuleDao,
    file::FileDao,
    project::ProjectDao,
    record::{RecordDao, RecordFilter, RecordFilters, RecordOrder, RecordPagination},
    token::TokenDao,
    value::ColumnValue,
    Db,
};

impl Db {
    pub async fn update_change(&self) -> Result<()> {
        hb_log::info(None, "[DAO] Updating changes table entries");

        let count_data_per_page = 30;

        hb_log::info(None, "[DAO] Updating changes table of admins data");
        let mut last_updated_at =
            match ChangeDao::db_select_last_by_table(self, &ChangeTable::Admin).await? {
                Some(change_data) => *change_data.updated_at(),
                None => DateTime::from_timestamp_millis(0).unwrap(),
            };
        let mut last_id = Uuid::nil();
        loop {
            let admins_data = AdminDao::db_select_many_from_updated_at_and_after_id_with_limit_asc(
                self,
                &last_updated_at,
                &last_id,
                &count_data_per_page,
            )
            .await?;
            match admins_data.last() {
                Some(admin_data) => {
                    last_updated_at = *admin_data.updated_at();
                    last_id = *admin_data.id();
                }
                None => break,
            }

            let mut changes_data = Vec::with_capacity(admins_data.len());
            for admin_data in &admins_data {
                changes_data.push(ChangeDao::new(
                    &ChangeTable::Admin,
                    admin_data.id(),
                    &ChangeState::Upsert,
                    admin_data.updated_at(),
                ))
            }
            let mut changes_data_fut = Vec::with_capacity(changes_data.len());
            for change_data in &changes_data {
                changes_data_fut.push(change_data.db_insert(self));
            }
            future::try_join_all(changes_data_fut).await?;
        }

        hb_log::info(None, "[DAO] Updating changes table of projects data");
        let mut last_updated_at =
            match ChangeDao::db_select_last_by_table(self, &ChangeTable::Project).await? {
                Some(change_data) => *change_data.updated_at(),
                None => DateTime::from_timestamp_millis(0).unwrap(),
            };
        let mut last_id = Uuid::nil();
        loop {
            let projects_data =
                ProjectDao::db_select_many_from_updated_at_and_after_id_with_limit_asc(
                    self,
                    &last_updated_at,
                    &last_id,
                    &count_data_per_page,
                )
                .await?;
            match projects_data.last() {
                Some(project_data) => {
                    last_updated_at = *project_data.updated_at();
                    last_id = *project_data.id();
                }
                None => break,
            }

            let mut changes_data = Vec::with_capacity(projects_data.len());
            for project_data in &projects_data {
                changes_data.push(ChangeDao::new(
                    &ChangeTable::Project,
                    project_data.id(),
                    &ChangeState::Upsert,
                    project_data.updated_at(),
                ));
            }
            let mut changes_data_fut = Vec::with_capacity(changes_data.len());
            for change_data in &changes_data {
                changes_data_fut.push(change_data.db_insert(self));
            }
            future::try_join_all(changes_data_fut).await?;
        }

        hb_log::info(None, "[DAO] Updating changes table of collections data");
        let mut last_updated_at = DateTime::from_timestamp_millis(0).unwrap();
        let mut last_id = Uuid::nil();
        loop {
            let collections_data =
                CollectionDao::db_select_many_from_updated_at_and_after_id_with_limit_asc(
                    self,
                    &last_updated_at,
                    &last_id,
                    &count_data_per_page,
                )
                .await?;
            match collections_data.last() {
                Some(collection_data) => {
                    last_updated_at = *collection_data.updated_at();
                    last_id = *collection_data.id();
                }
                None => break,
            }

            for collections_data in &collections_data {
                ChangeDao::new(
                    &ChangeTable::Collection,
                    collections_data.id(),
                    &ChangeState::Upsert,
                    collections_data.updated_at(),
                )
                .db_insert(self)
                .await?;

                hb_log::info(
                    None,
                    &format!(
                        "[DAO] Updating changes table of {} data",
                        RecordDao::new_table_name(collections_data.id())
                    ),
                );
                let mut last_updated_at = match ChangeDao::db_select_last_by_table(
                    self,
                    &ChangeTable::Record(*collections_data.id()),
                )
                .await?
                {
                    Some(change_data) => *change_data.updated_at(),
                    None => DateTime::from_timestamp_millis(0).unwrap(),
                };
                let mut last_id = Uuid::nil();
                loop {
                    let filters = RecordFilters::new(&vec![RecordFilter::new(
                        &None,
                        "OR",
                        &None,
                        &Some(RecordFilters::new(&vec![
                            RecordFilter::new(
                                &Some("_updated_at".to_owned()),
                                ">",
                                &Some(ColumnValue::Timestamp(Some(last_updated_at))),
                                &None,
                            ),
                            RecordFilter::new(
                                &None,
                                "AND",
                                &None,
                                &Some(RecordFilters::new(&vec![
                                    RecordFilter::new(
                                        &Some("_updated_at".to_owned()),
                                        "=",
                                        &Some(ColumnValue::Timestamp(Some(last_updated_at))),
                                        &None,
                                    ),
                                    RecordFilter::new(
                                        &Some("_id".to_owned()),
                                        ">",
                                        &Some(ColumnValue::Uuid(Some(last_id))),
                                        &None,
                                    ),
                                ])),
                            ),
                        ])),
                    )]);
                    let (records_data, _) = RecordDao::db_select_many(
                        self,
                        &HashSet::new(),
                        collections_data,
                        &None,
                        &filters,
                        &Vec::new(),
                        &vec![
                            RecordOrder::new("_updated_at", "ASC"),
                            RecordOrder::new("_id", "ASC"),
                        ],
                        &RecordPagination::new(&Some(count_data_per_page)),
                        &true,
                    )
                    .await?;
                    match records_data.last() {
                        Some(record_data) => {
                            last_updated_at =
                                if let Some(ColumnValue::Timestamp(Some(updated_at))) =
                                    record_data.get("_updated_at")
                                {
                                    *updated_at
                                } else {
                                    return Err(Error::msg(
                                        "Record _updated_at doesn't found".to_owned(),
                                    ));
                                };
                            last_id =
                                if let Some(ColumnValue::Uuid(Some(id))) = record_data.get("_id") {
                                    *id
                                } else {
                                    return Err(Error::msg("Record _id doesn't found".to_owned()));
                                }
                        }
                        None => break,
                    }

                    let mut changes_data = Vec::with_capacity(records_data.len());
                    for record_data in &records_data {
                        let id =
                            if let ColumnValue::Uuid(Some(id)) = record_data.get("_id").unwrap() {
                                id
                            } else {
                                return Err(Error::msg("Record id doesn't found".to_owned()));
                            };
                        let updated_at = if let ColumnValue::Timestamp(Some(updated_at)) =
                            record_data.get("_updated_at").unwrap()
                        {
                            updated_at
                        } else {
                            return Err(Error::msg("Record id doesn't found".to_owned()));
                        };
                        changes_data.push(ChangeDao::new(
                            &ChangeTable::Record(*record_data.collection_id()),
                            id,
                            &ChangeState::Upsert,
                            updated_at,
                        ));
                    }
                    let mut changes_data_fut = Vec::with_capacity(changes_data.len());
                    for change_data in &changes_data {
                        changes_data_fut.push(change_data.db_insert(self));
                    }
                    future::try_join_all(changes_data_fut).await?;
                }
            }
        }

        hb_log::info(None, "[DAO] Updating changes table of buckets data");
        let mut last_updated_at = DateTime::from_timestamp_millis(0).unwrap();
        let mut last_id = Uuid::nil();
        loop {
            let buckets_data =
                BucketDao::db_select_many_from_updated_at_and_after_id_with_limit_asc(
                    self,
                    &last_updated_at,
                    &last_id,
                    &count_data_per_page,
                )
                .await?;
            match buckets_data.last() {
                Some(bucket_data) => {
                    last_updated_at = *bucket_data.updated_at();
                    last_id = *bucket_data.id();
                }
                None => break,
            }

            for bucket_data in &buckets_data {
                ChangeDao::new(
                    &ChangeTable::Bucket,
                    bucket_data.id(),
                    &ChangeState::Upsert,
                    bucket_data.updated_at(),
                )
                .db_insert(self)
                .await?;

                hb_log::info(
                    None,
                    &format!(
                        "[DAO] Updating changes table of files data from bucket {}",
                        bucket_data.id()
                    ),
                );
                let mut last_updated_at = match ChangeDao::db_select_last_by_table(
                    self,
                    &ChangeTable::File(*bucket_data.id()),
                )
                .await?
                {
                    Some(change_data) => *change_data.updated_at(),
                    None => DateTime::from_timestamp_millis(0).unwrap(),
                };
                let mut last_id = Uuid::nil();
                loop {
                    let files_data =
                        FileDao::db_select_many_from_updated_at_and_after_id_with_limit_asc(
                            self,
                            &last_updated_at,
                            &last_id,
                            &count_data_per_page,
                        )
                        .await?;
                    match files_data.last() {
                        Some(file_data) => {
                            last_updated_at = *file_data.updated_at();
                            last_id = *file_data.id();
                        }
                        None => break,
                    }

                    let mut changes_data = Vec::with_capacity(files_data.len());
                    for file_data in &files_data {
                        changes_data.push(ChangeDao::new(
                            &ChangeTable::File(*file_data.bucket_id()),
                            file_data.id(),
                            &ChangeState::Upsert,
                            file_data.updated_at(),
                        ));
                    }
                    let mut changes_data_fut = Vec::with_capacity(changes_data.len());
                    for change_data in &changes_data {
                        changes_data_fut.push(change_data.db_insert(self));
                    }
                    future::try_join_all(changes_data_fut).await?;
                }
            }
        }

        hb_log::info(None, "[DAO] Updating changes table of tokens data");
        let mut last_updated_at =
            match ChangeDao::db_select_last_by_table(self, &ChangeTable::Token).await? {
                Some(change_data) => *change_data.updated_at(),
                None => DateTime::from_timestamp_millis(0).unwrap(),
            };
        let mut last_id = Uuid::nil();
        loop {
            let tokens_data = TokenDao::db_select_many_from_updated_at_and_after_id_with_limit_asc(
                self,
                &last_updated_at,
                &last_id,
                &count_data_per_page,
            )
            .await?;
            match tokens_data.last() {
                Some(token_data) => {
                    last_updated_at = *token_data.updated_at();
                    last_id = *token_data.id();
                }
                None => break,
            }

            let mut changes_data = Vec::with_capacity(tokens_data.len());
            for token_data in &tokens_data {
                changes_data.push(ChangeDao::new(
                    &ChangeTable::Token,
                    token_data.id(),
                    &ChangeState::Upsert,
                    token_data.updated_at(),
                ));
            }
            let mut changes_data_fut = Vec::with_capacity(changes_data.len());
            for change_data in &changes_data {
                changes_data_fut.push(change_data.db_insert(self));
            }
            future::try_join_all(changes_data_fut).await?;
        }

        hb_log::info(
            None,
            "[DAO] Updating changes table of collection_rules data",
        );
        let mut last_updated_at =
            match ChangeDao::db_select_last_by_table(self, &ChangeTable::CollectionRule).await? {
                Some(change_data) => *change_data.updated_at(),
                None => DateTime::from_timestamp_millis(0).unwrap(),
            };
        let mut last_id = Uuid::nil();
        loop {
            let collection_rules_data =
                CollectionRuleDao::db_select_many_from_updated_at_and_after_id_with_limit_asc(
                    self,
                    &last_updated_at,
                    &last_id,
                    &count_data_per_page,
                )
                .await?;
            match collection_rules_data.last() {
                Some(collection_rule) => {
                    last_updated_at = *collection_rule.updated_at();
                    last_id = *collection_rule.id();
                }
                None => break,
            }

            let mut changes_data = Vec::with_capacity(collection_rules_data.len());
            for collection_rule in &collection_rules_data {
                changes_data.push(ChangeDao::new(
                    &ChangeTable::CollectionRule,
                    collection_rule.id(),
                    &ChangeState::Upsert,
                    collection_rule.updated_at(),
                ));
            }
            let mut changes_data_fut = Vec::with_capacity(changes_data.len());
            for change_data in &changes_data {
                changes_data_fut.push(change_data.db_insert(self));
            }
            future::try_join_all(changes_data_fut).await?;
        }

        hb_log::info(None, "[DAO] Updating changes table of bucket_rules data");

        let mut last_updated_at =
            match ChangeDao::db_select_last_by_table(self, &ChangeTable::BucketRule).await? {
                Some(change_data) => *change_data.updated_at(),
                None => DateTime::from_timestamp_millis(0).unwrap(),
            };
        let mut last_id = Uuid::nil();
        loop {
            let bucket_rules_data =
                BucketRuleDao::db_select_many_from_updated_at_and_after_id_with_limit_asc(
                    self,
                    &last_updated_at,
                    &last_id,
                    &count_data_per_page,
                )
                .await?;
            match bucket_rules_data.last() {
                Some(bucket_rule) => {
                    last_updated_at = *bucket_rule.updated_at();
                    last_id = *bucket_rule.id();
                }
                None => break,
            }

            let mut changes_data = Vec::with_capacity(bucket_rules_data.len());
            for bucket_rule in &bucket_rules_data {
                changes_data.push(ChangeDao::new(
                    &ChangeTable::BucketRule,
                    bucket_rule.id(),
                    &ChangeState::Upsert,
                    bucket_rule.updated_at(),
                ));
            }
            let mut changes_data_fut = Vec::with_capacity(changes_data.len());
            for change_data in &changes_data {
                changes_data_fut.push(change_data.db_insert(self));
            }
            future::try_join_all(changes_data_fut).await?;
        }

        Ok(())
    }
}
