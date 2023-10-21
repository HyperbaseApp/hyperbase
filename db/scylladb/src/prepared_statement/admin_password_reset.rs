use scylla::{prepared_statement::PreparedStatement, Session};

pub struct AdminPasswordResetPreparedStatement {
    insert: PreparedStatement,
    select: PreparedStatement,
    update: PreparedStatement,
    delete: PreparedStatement,
}

impl AdminPasswordResetPreparedStatement {
    pub async fn new(session: &Session) -> Self {
        Self {
            insert: session.prepare(format!("INSERT INTO {} (\"id\", \"created_at\", \"updated_at\", \"email\", \"code\") VALUES (?, ?, ?, ?, ?)", Self::table_name())).await.unwrap(),
            select: session.prepare(format!("SELECT * FROM {} WHERE \"id\" = ?", Self::table_name())).await.unwrap(),
            update: session.prepare(format!("UPDATE {} SET \"updated_at\" = ?, \"code\" = ? WHERE \"id\" = ?", Self::table_name())).await.unwrap(),
            delete: session.prepare(format!("DELETE FROM {} WHERE \"id\" = ?", Self::table_name())).await.unwrap(),
        }
    }

    pub fn table_name() -> &'static str {
        "ks.admin_password_resets"
    }

    pub fn insert(&self) -> &PreparedStatement {
        &self.insert
    }

    pub fn select(&self) -> &PreparedStatement {
        &self.select
    }

    pub fn update(&self) -> &PreparedStatement {
        &self.update
    }

    pub fn delete(&self) -> &PreparedStatement {
        &self.delete
    }
}
