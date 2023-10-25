use scylla::{prepared_statement::PreparedStatement, Session};

pub struct TokenPreparedStatement {
    insert: PreparedStatement,
    select: PreparedStatement,
    select_by_token: PreparedStatement,
    update: PreparedStatement,
    delete: PreparedStatement,
}

impl TokenPreparedStatement {
    pub async fn new(session: &Session) -> Self {
        Self{
            insert: session.prepare(format!("INSERT INTO {} (\"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"expired_at\") VALUES (?, ?, ?, ?, ?, ?)", Self::table_name())).await.unwrap(),
            select: session.prepare(format!("SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"expired_at\" FROM {} WHERE \"id\" = ?", Self::table_name())).await.unwrap(),
            select_by_token: session.prepare(format!("SELECT \"id\", \"created_at\", \"updated_at\", \"admin_id\", \"token\", \"expired_at\" FROM {} WHERE \"token\" = ?", Self::table_name())).await.unwrap(),
            update: session.prepare(format!("UPDATE {} SET \"updated_at\" = ?, \"expired_at\" = ? WHERE \"id\" = ?", Self::table_name())).await.unwrap(),
            delete: session.prepare(format!("DELETE FROM {} WHERE \"id\" = ?", Self::table_name())).await.unwrap(),
        }
    }

    pub fn table_name() -> &'static str {
        "ks.tokens"
    }

    pub fn insert(&self) -> &PreparedStatement {
        &self.insert
    }

    pub fn select(&self) -> &PreparedStatement {
        &self.select
    }

    pub fn select_by_token(&self) -> &PreparedStatement {
        &self.select_by_token
    }

    pub fn update(&self) -> &PreparedStatement {
        &self.update
    }

    pub fn delete(&self) -> &PreparedStatement {
        &self.delete
    }
}
