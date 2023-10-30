use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_scylladb::{db::ScyllaDb, model::project::ProjectScyllaModel};
use scylla::{frame::value::Timestamp, transport::session::TypedRowIter};
use uuid::Uuid;

use crate::{
    util::conversion::{datetime_to_duration_since_epoch, duration_since_epoch_to_datetime},
    Db,
};

pub struct ProjectDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    admin_id: Uuid,
    name: String,
}

impl ProjectDao {
    pub fn new(admin_id: &Uuid, name: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            admin_id: *admin_id,
            name: name.to_string(),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
}

impl ProjectDao {
    pub async fn insert(&self, db: &Db<'_>) -> Result<()> {
        match *db {
            Db::ScyllaDb(db) => Self::scylladb_insert(&self, db).await,
        }
    }

    pub async fn select(db: &Db<'_>, id: &Uuid) -> Result<Self> {
        match *db {
            Db::ScyllaDb(db) => Ok(Self::from_scylladb_model(
                &Self::scylladb_select(db, id).await?,
            )?),
        }
    }

    pub async fn select_many_by_admin_id(db: &Db<'_>, admin_id: &Uuid) -> Result<Vec<Self>> {
        match *db {
            Db::ScyllaDb(db) => {
                let mut projects_data = Vec::new();
                let projects = Self::scylladb_select_many_by_admin_id(db, admin_id).await?;
                for project in projects {
                    if let Ok(model) = &project {
                        projects_data.push(Self::from_scylladb_model(model)?);
                    } else if let Err(err) = project {
                        return Err(err.into());
                    }
                }
                Ok(projects_data)
            }
        }
    }

    pub async fn update(&mut self, db: &Db<'_>) -> Result<()> {
        self.updated_at = Utc::now();
        match *db {
            Db::ScyllaDb(db) => Self::scylladb_update(&self, db).await,
        }
    }

    pub async fn delete(db: &Db<'_>, id: &Uuid) -> Result<()> {
        match *db {
            Db::ScyllaDb(db) => Self::scylladb_delete(db, id).await,
        }
    }
}

impl ProjectDao {
    async fn scylladb_insert(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(
            db.prepared_statement().project().insert(),
            self.to_scylladb_model(),
        )
        .await?;
        Ok(())
    }

    async fn scylladb_select(db: &ScyllaDb, id: &Uuid) -> Result<ProjectScyllaModel> {
        Ok(db
            .execute(db.prepared_statement().project().select(), [id].as_ref())
            .await?
            .first_row_typed::<ProjectScyllaModel>()?)
    }

    async fn scylladb_select_many_by_admin_id(
        db: &ScyllaDb,
        admin_id: &Uuid,
    ) -> Result<TypedRowIter<ProjectScyllaModel>> {
        Ok(db
            .execute(
                db.prepared_statement().project().select_many_by_admin_id(),
                [admin_id].as_ref(),
            )
            .await?
            .rows_typed::<ProjectScyllaModel>()?)
    }

    async fn scylladb_update(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(
            db.prepared_statement().project().update(),
            (&self.updated_at, &self.name, &self.id),
        )
        .await?;
        Ok(())
    }

    async fn scylladb_delete(db: &ScyllaDb, id: &Uuid) -> Result<()> {
        db.execute(db.prepared_statement().project().delete(), [id].as_ref())
            .await?;
        Ok(())
    }

    fn from_scylladb_model(model: &ProjectScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: duration_since_epoch_to_datetime(model.created_at().0)?,
            updated_at: duration_since_epoch_to_datetime(model.updated_at().0)?,
            admin_id: *model.admin_id(),
            name: model.name().to_string(),
        })
    }

    fn to_scylladb_model(&self) -> ProjectScyllaModel {
        ProjectScyllaModel::new(
            &self.id,
            &Timestamp(datetime_to_duration_since_epoch(self.created_at)),
            &Timestamp(datetime_to_duration_since_epoch(self.updated_at)),
            &self.admin_id,
            &self.name,
        )
    }
}
