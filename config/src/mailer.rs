use serde::Deserialize;

#[derive(Deserialize)]
pub struct MailerConfig {
    smtp_host: String,
    smtp_username: String,
    smtp_password: String,
    sender_name: String,
    sender_email: String,
}

impl MailerConfig {
    pub fn smtp_host(&self) -> &str {
        &self.smtp_host
    }

    pub fn smtp_username(&self) -> &str {
        &self.smtp_username
    }

    pub fn smtp_password(&self) -> &str {
        &self.smtp_password
    }

    pub fn sender_name(&self) -> &str {
        &self.sender_name
    }

    pub fn sender_email(&self) -> &str {
        &self.sender_email
    }
}
