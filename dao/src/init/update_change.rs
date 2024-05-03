use anyhow::Result;

use crate::{
    admin::AdminDao,
    bucket::BucketDao,
    bucket_rule::BucketRuleDao,
    change::{ChangeDao, ChangeState, ChangeTable},
    collection::CollectionDao,
    collection_rule::CollectionRuleDao,
    file::FileDao,
    project::ProjectDao,
    token::TokenDao,
    Db,
};

impl Db {
    pub async fn update_change(&self) -> Result<()> {
        hb_log::info(None, "[DAO] Updating changes table entries");

        let count_data_per_page = 30;

        match ChangeDao::db_select_last(self).await? {
            Some(_) => {}
            None => {
                let mut last_admin_id = None;
                loop {
                    let admins_data = AdminDao::db_select_many_after_id_with_limit(
                        self,
                        &last_admin_id,
                        &count_data_per_page,
                    )
                    .await?;
                    match admins_data.last() {
                        Some(admin_data) => {
                            last_admin_id = Some(*admin_data.id());
                        }
                        None => break,
                    }
                    for admin_data in &admins_data {
                        ChangeDao::new(
                            &ChangeTable::Admin,
                            admin_data.id(),
                            &ChangeState::Insert,
                            admin_data.updated_at(),
                        )
                        .db_insert(self)
                        .await?;
                    }
                }

                let mut last_project_id = None;
                loop {
                    let projects_data = ProjectDao::db_select_many_after_id_with_limit(
                        self,
                        &last_project_id,
                        &count_data_per_page,
                    )
                    .await?;
                    match projects_data.last() {
                        Some(project_data) => {
                            last_project_id = Some(*project_data.id());
                        }
                        None => break,
                    }
                    for project_data in &projects_data {
                        ChangeDao::new(
                            &ChangeTable::Project,
                            project_data.id(),
                            &ChangeState::Insert,
                            project_data.updated_at(),
                        )
                        .db_insert(self)
                        .await?;
                    }
                }

                let mut last_collection_id = None;
                loop {
                    let collections_data = CollectionDao::db_select_many_after_id_with_limit(
                        self,
                        &last_collection_id,
                        &count_data_per_page,
                    )
                    .await?;
                    match collections_data.last() {
                        Some(collection_data) => {
                            last_collection_id = Some(*collection_data.id());
                        }
                        None => break,
                    }
                    for collections_data in &collections_data {
                        ChangeDao::new(
                            &ChangeTable::Collection,
                            collections_data.id(),
                            &ChangeState::Insert,
                            collections_data.updated_at(),
                        )
                        .db_insert(self)
                        .await?;
                    }
                }

                let mut last_bucket_id = None;
                loop {
                    let buckets_data = BucketDao::db_select_many_after_id_with_limit(
                        self,
                        &last_bucket_id,
                        &count_data_per_page,
                    )
                    .await?;
                    match buckets_data.last() {
                        Some(bucket_data) => {
                            last_bucket_id = Some(*bucket_data.id());
                        }
                        None => break,
                    }
                    for bucket_data in &buckets_data {
                        ChangeDao::new(
                            &ChangeTable::Bucket,
                            bucket_data.id(),
                            &ChangeState::Insert,
                            bucket_data.updated_at(),
                        )
                        .db_insert(self)
                        .await?;
                    }
                }

                let mut last_file_id = None;
                loop {
                    let files_data = FileDao::db_select_many_after_id_with_limit(
                        self,
                        &last_file_id,
                        &count_data_per_page,
                    )
                    .await?;
                    match files_data.last() {
                        Some(file_data) => {
                            last_file_id = Some(*file_data.id());
                        }
                        None => break,
                    }
                    for file_data in &files_data {
                        ChangeDao::new(
                            &ChangeTable::File,
                            file_data.id(),
                            &ChangeState::Insert,
                            file_data.updated_at(),
                        )
                        .db_insert(self)
                        .await?;
                    }
                }

                let mut last_token_id = None;
                loop {
                    let tokens_data = TokenDao::db_select_many_after_id_with_limit(
                        self,
                        &last_token_id,
                        &count_data_per_page,
                    )
                    .await?;
                    match tokens_data.last() {
                        Some(token_data) => {
                            last_token_id = Some(*token_data.id());
                        }
                        None => break,
                    }
                    for token_data in &tokens_data {
                        ChangeDao::new(
                            &ChangeTable::Token,
                            token_data.id(),
                            &ChangeState::Insert,
                            token_data.updated_at(),
                        )
                        .db_insert(self)
                        .await?;
                    }
                }

                let mut last_collection_rule_id = None;
                loop {
                    let collection_rules_data =
                        CollectionRuleDao::db_select_many_after_id_with_limit(
                            self,
                            &last_collection_rule_id,
                            &count_data_per_page,
                        )
                        .await?;
                    match collection_rules_data.last() {
                        Some(collection_rule) => {
                            last_collection_rule_id = Some(*collection_rule.id());
                        }
                        None => break,
                    }
                    for collection_rule in &collection_rules_data {
                        ChangeDao::new(
                            &ChangeTable::CollectionRule,
                            collection_rule.id(),
                            &ChangeState::Insert,
                            collection_rule.updated_at(),
                        )
                        .db_insert(self)
                        .await?;
                    }
                }

                let mut last_bucket_rule_id = None;
                loop {
                    let bucket_rules_data = BucketRuleDao::db_select_many_after_id_with_limit(
                        self,
                        &last_bucket_rule_id,
                        &count_data_per_page,
                    )
                    .await?;
                    match bucket_rules_data.last() {
                        Some(bucket_rule) => {
                            last_bucket_rule_id = Some(*bucket_rule.id());
                        }
                        None => break,
                    }
                    for bucket_rule in &bucket_rules_data {
                        ChangeDao::new(
                            &ChangeTable::BucketRule,
                            bucket_rule.id(),
                            &ChangeState::Insert,
                            bucket_rule.updated_at(),
                        )
                        .db_insert(self)
                        .await?;
                    }
                }
            }
        }

        Ok(())
    }
}
