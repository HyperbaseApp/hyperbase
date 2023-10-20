use scylla::{prepared_statement::PreparedStatement, Session};

pub struct CollectionPreparedStatement {
    insert: PreparedStatement,
    select: PreparedStatement,
    update: PreparedStatement,
    delete: PreparedStatement,
}

impl CollectionPreparedStatement {
    pub async fn new(session: &Session) -> Self {
        Self{
            insert: session.prepare(format!("INSERT INTO {} (\"id\", \"created_at\", \"updated_at\", \"project_id\", \"name\", \"schema_fields\", \"indexes\") VALUES (?, ?, ?, ?, ?, ?, ?)", Self::table_name())).await.unwrap(),    
            select: session.prepare(format!("SELECT * FROM {} WHERE \"id\" = ?", Self::table_name())).await.unwrap(),
            update: session.prepare(format!("UPDATE {} SET \"updated_at\" = ?, \"name\" = ?, \"schema_fields\" = ?, \"indexes\" = ? WHERE \"id\" = ?", Self::table_name())).await.unwrap(),
            delete: session.prepare(format!("DELETE FROM {} WHERE \"id\" = ?", Self::table_name())).await.unwrap(),
        }
    }

    pub fn table_name() -> &'static str {
        "ks.collections"
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
