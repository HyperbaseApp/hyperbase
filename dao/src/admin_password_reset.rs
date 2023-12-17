use anyhow::Result;
use chrono::{DateTime, Utc};
use hb_db_mysql::{
    db::MysqlDb,
    model::admin_password_reset::AdminPasswordResetModel as AdminPasswordResetMysqlModel,
    query::admin_password_reset::{INSERT as MYSQL_INSERT, SELECT as MYSQL_SELECT},
};
use hb_db_postgresql::{
    db::PostgresDb,
    model::admin_password_reset::AdminPasswordResetModel as AdminPasswordResetPostgresModel,
    query::admin_password_reset::{INSERT as POSTGRES_INSERT, SELECT as POSTGRES_SELECT},
};
use hb_db_scylladb::{
    db::ScyllaDb,
    model::admin_password_reset::AdminPasswordResetModel as AdminPasswordResetScyllaModel,
    query::admin_password_reset::{INSERT as SCYLLA_INSERT, SELECT as SCYLLA_SELECT},
};
use hb_db_sqlite::{
    db::SqliteDb,
    model::admin_password_reset::AdminPasswordResetModel as AdminPasswordResetSqliteModel,
    query::admin_password_reset::{INSERT as SQLITE_INSERT, SELECT as SQLITE_SELECT},
};
use rand::{thread_rng, Rng};
use scylla::frame::value::Timestamp;
use uuid::Uuid;

use crate::{
    util::conversion::{datetime_to_duration_since_epoch, duration_since_epoch_to_datetime},
    Db,
};

pub struct AdminPasswordResetDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    admin_id: Uuid,
    code: String,
}

impl AdminPasswordResetDao {
    pub fn new(admin_id: &Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            admin_id: *admin_id,
            code: thread_rng().gen_range(100000..=999999).to_string(),
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

    pub fn code(&self) -> &str {
        &self.code
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => Self::scylladb_insert(self, db).await,
            Db::PostgresqlDb(db) => Self::postgresdb_insert(self, db).await,
            Db::MysqlDb(db) => Self::mysqldb_insert(self, db).await,
            Db::SqliteDb(db) => Self::sqlitedb_insert(self, db).await,
        }
    }

    pub async fn db_select(db: &Db, id: &Uuid) -> Result<Self> {
        match db {
            Db::ScyllaDb(db) => Ok(Self::from_scylladb_model(
                &Self::scylladb_select(db, id).await?,
            )?),
            Db::PostgresqlDb(db) => Ok(Self::from_postgresdb_model(
                &Self::postgresdb_select(db, id).await?,
            )?),
            Db::MysqlDb(db) => Ok(Self::from_mysqldb_model(
                &Self::mysqldb_select(db, id).await?,
            )?),
            Db::SqliteDb(db) => Ok(Self::from_sqlitedb_model(
                &Self::sqlitedb_select(db, id).await?,
            )?),
        }
    }

    async fn scylladb_insert(&self, db: &ScyllaDb) -> Result<()> {
        db.execute(SCYLLA_INSERT, &self.to_scylladb_model()).await?;
        Ok(())
    }

    async fn scylladb_select(db: &ScyllaDb, id: &Uuid) -> Result<AdminPasswordResetScyllaModel> {
        Ok(db
            .execute(SCYLLA_SELECT, [id].as_ref())
            .await?
            .first_row_typed::<AdminPasswordResetScyllaModel>()?)
    }

    async fn postgresdb_insert(&self, db: &PostgresDb) -> Result<()> {
        db.execute(
            sqlx::query(POSTGRES_INSERT)
                .bind(&self.id)
                .bind(&self.created_at)
                .bind(&self.updated_at)
                .bind(&self.admin_id)
                .bind(&self.code),
        )
        .await?;
        Ok(())
    }

    async fn postgresdb_select(
        db: &PostgresDb,
        id: &Uuid,
    ) -> Result<AdminPasswordResetPostgresModel> {
        Ok(db
            .fetch_one::<AdminPasswordResetPostgresModel>(
                sqlx::query_as(POSTGRES_SELECT)
                    .bind(id)
                    .bind(db.table_reset_password_ttl()),
            )
            .await?)
    }

    async fn mysqldb_insert(&self, db: &MysqlDb) -> Result<()> {
        db.execute(
            sqlx::query(MYSQL_INSERT)
                .bind(&self.id)
                .bind(&self.created_at)
                .bind(&self.updated_at)
                .bind(&self.admin_id)
                .bind(&self.code),
        )
        .await?;
        Ok(())
    }

    async fn mysqldb_select(db: &MysqlDb, id: &Uuid) -> Result<AdminPasswordResetMysqlModel> {
        Ok(db
            .fetch_one::<AdminPasswordResetMysqlModel>(
                sqlx::query_as(MYSQL_SELECT)
                    .bind(id)
                    .bind(db.table_reset_password_ttl()),
            )
            .await?)
    }

    async fn sqlitedb_insert(&self, db: &SqliteDb) -> Result<()> {
        db.execute(
            sqlx::query(SQLITE_INSERT)
                .bind(&self.id)
                .bind(&self.created_at)
                .bind(&self.updated_at)
                .bind(&self.admin_id)
                .bind(&self.code),
        )
        .await?;
        Ok(())
    }

    async fn sqlitedb_select(db: &SqliteDb, id: &Uuid) -> Result<AdminPasswordResetSqliteModel> {
        Ok(db
            .fetch_one::<AdminPasswordResetSqliteModel>(
                sqlx::query_as(SQLITE_SELECT)
                    .bind(id)
                    .bind(db.table_reset_password_ttl()),
            )
            .await?)
    }

    fn from_scylladb_model(model: &AdminPasswordResetScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: duration_since_epoch_to_datetime(&model.created_at().0)?,
            updated_at: duration_since_epoch_to_datetime(&model.updated_at().0)?,
            admin_id: *model.admin_id(),
            code: model.code().to_owned(),
        })
    }

    fn to_scylladb_model(&self) -> AdminPasswordResetScyllaModel {
        AdminPasswordResetScyllaModel::new(
            &self.id,
            &Timestamp(datetime_to_duration_since_epoch(&self.created_at)),
            &Timestamp(datetime_to_duration_since_epoch(&self.updated_at)),
            &self.admin_id,
            &self.code,
        )
    }

    fn from_postgresdb_model(model: &AdminPasswordResetPostgresModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            admin_id: *model.admin_id(),
            code: model.code().to_owned(),
        })
    }

    fn from_mysqldb_model(model: &AdminPasswordResetMysqlModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            admin_id: *model.admin_id(),
            code: model.code().to_owned(),
        })
    }

    fn from_sqlitedb_model(model: &AdminPasswordResetSqliteModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            updated_at: *model.updated_at(),
            admin_id: *model.admin_id(),
            code: model.code().to_owned(),
        })
    }
}
