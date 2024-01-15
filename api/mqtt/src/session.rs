use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Session {
    client_id: String,
    token_id: Uuid,
}

impl Session {
    pub fn new(client_id: &str, token_id: &Uuid) -> Self {
        Self {
            client_id: client_id.to_owned(),
            token_id: *token_id,
        }
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn token_id(&self) -> &Uuid {
        &self.token_id
    }
}
