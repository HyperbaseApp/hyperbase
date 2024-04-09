use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use hb_db_mysql::model::log::LogModel as LogMysqlModel;
use hb_db_postgresql::model::log::LogModel as LogPostgresModel;
use hb_db_scylladb::model::log::LogModel as LogScyllaModel;
use hb_db_sqlite::model::log::LogModel as LogSqliteModel;
use scylla::frame::value::CqlTimestamp as ScyllaCqlTimestamp;
use uuid::Uuid;

use crate::{util::conversion, Db};

pub struct LogDao {
    id: Uuid,
    created_at: DateTime<Utc>,
    admin_id: Uuid,
    project_id: Uuid,
    kind: LogKind,
    message: String,
}

impl LogDao {
    pub fn new(admin_id: &Uuid, project_id: &Uuid, kind: &LogKind, message: &str) -> Self {
        Self {
            id: Uuid::now_v7(),
            created_at: Utc::now(),
            admin_id: *admin_id,
            project_id: *project_id,
            kind: *kind,
            message: message.to_owned(),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn admin_id(&self) -> &Uuid {
        &self.admin_id
    }

    pub fn project_id(&self) -> &Uuid {
        &self.project_id
    }

    pub fn kind(&self) -> &LogKind {
        &self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub async fn db_insert(&self, db: &Db) -> Result<()> {
        match db {
            Db::ScyllaDb(db) => db.insert_log(&self.to_scylladb_model()).await,
            Db::PostgresqlDb(db) => db.insert_log(&self.to_postgresdb_model()).await,
            Db::MysqlDb(db) => db.insert_log(&self.to_mysqldb_model()).await,
            Db::SqliteDb(db) => db.insert_log(&self.to_sqlitedb_model()).await,
        }
    }

    pub async fn db_select_many_by_admin_id_and_project_id(
        db: &Db,
        admin_id: &Uuid,
        project_id: &Uuid,
        before_id: &Option<Uuid>,
        limit: &Option<i32>,
    ) -> Result<(Vec<Self>, i64)> {
        match db {
            Db::ScyllaDb(db) => {
                let mut logs_data = Vec::new();
                let (logs, total) = tokio::try_join!(
                    db.select_many_logs_by_admin_id_and_project_id(
                        admin_id, project_id, before_id, limit
                    ),
                    db.count_many_logs_by_admin_id_and_project_id(admin_id, project_id)
                )?;
                for log in logs {
                    logs_data.push(Self::from_scylladb_model(&log?)?);
                }
                Ok((logs_data, total))
            }
            Db::PostgresqlDb(db) => {
                let (logs, total) = tokio::try_join!(
                    db.select_many_logs_by_admin_id_and_project_id(
                        admin_id, project_id, before_id, limit
                    ),
                    db.count_many_logs_by_admin_id_and_project_id(admin_id, project_id)
                )?;
                let mut logs_data = Vec::with_capacity(logs.len());
                for log in &logs {
                    logs_data.push(Self::from_postgresdb_model(log)?);
                }
                Ok((logs_data, total))
            }
            Db::MysqlDb(db) => {
                let (logs, total) = tokio::try_join!(
                    db.select_many_logs_by_admin_id_and_project_id(
                        admin_id, project_id, before_id, limit
                    ),
                    db.count_many_logs_by_admin_id_and_project_id(admin_id, project_id)
                )?;
                let mut logs_data = Vec::with_capacity(logs.len());
                for log in &logs {
                    logs_data.push(Self::from_mysqldb_model(log)?);
                }
                Ok((logs_data, total))
            }
            Db::SqliteDb(db) => {
                let (logs, total) = tokio::try_join!(
                    db.select_many_logs_by_admin_id_and_project_id(
                        admin_id, project_id, before_id, limit
                    ),
                    db.count_many_logs_by_admin_id_and_project_id(admin_id, project_id)
                )?;
                let mut logs_data = Vec::with_capacity(logs.len());
                for log in &logs {
                    logs_data.push(Self::from_sqlitedb_model(log)?);
                }
                Ok((logs_data, total))
            }
        }
    }

    fn from_scylladb_model(model: &LogScyllaModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: conversion::scylla_cql_timestamp_to_datetime_utc(model.created_at())?,
            admin_id: *model.admin_id(),
            project_id: *model.project_id(),
            kind: LogKind::from_str(model.kind())?,
            message: model.message().to_owned(),
        })
    }

    fn to_scylladb_model(&self) -> LogScyllaModel {
        LogScyllaModel::new(
            &self.id,
            &ScyllaCqlTimestamp(self.created_at.timestamp_millis()),
            &self.admin_id,
            &self.project_id,
            self.kind.to_str(),
            &self.message,
        )
    }

    fn from_postgresdb_model(model: &LogPostgresModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            admin_id: *model.admin_id(),
            project_id: *model.project_id(),
            kind: LogKind::from_str(model.kind())?,
            message: model.message().to_owned(),
        })
    }

    fn to_postgresdb_model(&self) -> LogPostgresModel {
        LogPostgresModel::new(
            &self.id,
            &self.created_at,
            &self.admin_id,
            &self.project_id,
            self.kind.to_str(),
            &self.message,
        )
    }

    fn from_mysqldb_model(model: &LogMysqlModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            admin_id: *model.admin_id(),
            project_id: *model.project_id(),
            kind: LogKind::from_str(model.kind())?,
            message: model.message().to_owned(),
        })
    }

    fn to_mysqldb_model(&self) -> LogMysqlModel {
        LogMysqlModel::new(
            &self.id,
            &self.created_at,
            &self.admin_id,
            &self.project_id,
            self.kind.to_str(),
            &self.message,
        )
    }

    fn from_sqlitedb_model(model: &LogSqliteModel) -> Result<Self> {
        Ok(Self {
            id: *model.id(),
            created_at: *model.created_at(),
            admin_id: *model.admin_id(),
            project_id: *model.project_id(),
            kind: LogKind::from_str(model.kind())?,
            message: model.message().to_owned(),
        })
    }

    fn to_sqlitedb_model(&self) -> LogSqliteModel {
        LogSqliteModel::new(
            &self.id,
            &self.created_at,
            &self.admin_id,
            &self.project_id,
            self.kind.to_str(),
            &self.message,
        )
    }
}

#[derive(Clone, Copy)]
pub enum LogKind {
    Error, // Designates very serious errors.
    Warn,  // Designates hazardous situations.
    Info,  // Designates useful information.
    Debug, // Designates lower priority information.
    Trace, // Designates very low priority, often extremely verbose, information.
}

impl LogKind {
    pub fn from_str(str: &str) -> Result<Self> {
        match str {
            "error" => Ok(Self::Error),
            "warn" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            "trace" => Ok(Self::Trace),
            _ => Err(Error::msg(format!("Unknown log kind '{str}'"))),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}
