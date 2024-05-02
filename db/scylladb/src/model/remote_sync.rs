use scylla::{frame::value::CqlTimestamp, FromRow, SerializeRow};
use uuid::Uuid;

#[derive(FromRow, SerializeRow)]
pub struct RemoteSyncModel {
    remote_address: String,
    remote_id: Uuid,
    last_data_sync: CqlTimestamp,
}

impl RemoteSyncModel {
    pub fn new(remote_address: &str, remote_id: &Uuid, last_data_sync: &CqlTimestamp) -> Self {
        Self {
            remote_address: remote_address.to_owned(),
            remote_id: *remote_id,
            last_data_sync: *last_data_sync,
        }
    }

    pub fn remote_address(&self) -> &str {
        &self.remote_address
    }

    pub fn remote_id(&self) -> &Uuid {
        &self.remote_id
    }

    pub fn last_data_sync(&self) -> &CqlTimestamp {
        &self.last_data_sync
    }
}
