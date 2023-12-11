use uuid::Uuid;

pub struct Pagination {
    last_id: Option<Uuid>,
    limit: Option<i32>,
}

impl Pagination {
    pub fn new(last_id: &Option<Uuid>, limit: &Option<i32>) -> Self {
        Self {
            last_id: *last_id,
            limit: *limit,
        }
    }

    pub fn last_id(&self) -> &Option<Uuid> {
        &self.last_id
    }

    pub fn limit(&self) -> &Option<i32> {
        &self.limit
    }
}
