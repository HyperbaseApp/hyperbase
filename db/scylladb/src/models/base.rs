pub struct ScyllaBaseModel {
    pub(super) id: uuid::Uuid,
    pub(super) created_at: std::time::Duration,
    pub(super) updated_at: std::time::Duration,
}
