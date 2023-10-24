use scylla::{prepared_statement::PreparedStatement, Session};

pub struct AdminPreparedStatement {
    insert: PreparedStatement,
    select: PreparedStatement,
    select_by_email: PreparedStatement,
    update: PreparedStatement,
    delete: PreparedStatement,
}

impl AdminPreparedStatement {
    pub async fn new(session: &Session) -> Self {
        Self {
            insert: session.prepare(format!("INSERT INTO {} (\"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\") VALUES (?, ?, ?, ?, ?)", Self::table_name())).await.unwrap(),
            select: session.prepare(format!("SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\" FROM {} WHERE \"id\" = ?", Self::table_name())).await.unwrap(),
            select_by_email: session.prepare(format!("SELECT \"id\", \"created_at\", \"updated_at\", \"email\", \"password_hash\" FROM {} WHERE \"email\" = ?", Self::table_name())).await.unwrap(),
            update: session.prepare(format!("UPDATE {} SET \"updated_at\" = ?, \"email\" = ?, \"password_hash\" = ? WHERE \"id\" = ?", Self::table_name())).await.unwrap(),
            delete: session.prepare(format!("DELETE FROM {} WHERE \"id\" = ?", Self::table_name())).await.unwrap(),
        }
    }

    pub fn table_name() -> &'static str {
        "ks.admins"
    }

    pub fn insert(&self) -> &PreparedStatement {
        &self.insert
    }

    pub fn select(&self) -> &PreparedStatement {
        &self.select
    }

    pub fn select_by_email(&self) -> &PreparedStatement {
        &self.select_by_email
    }

    pub fn update(&self) -> &PreparedStatement {
        &self.update
    }

    pub fn delete(&self) -> &PreparedStatement {
        &self.delete
    }
}
