use anyhow::Result;
use lettre::{
    message::MessageBuilder, transport::smtp::authentication::Credentials, Message, SmtpTransport,
    Transport,
};

pub struct Mailer {
    message_builder: MessageBuilder,
    smtp_transport: SmtpTransport,
}

impl Mailer {
    pub fn new(
        smtp_server: &str,
        smtp_username: &str,
        smtp_password: &str,
        sender_name: &str,
        sender_email: &str,
    ) -> Self {
        Self {
            message_builder: Message::builder()
                .from(format!("{sender_name} <{sender_email}>").parse().unwrap()),
            smtp_transport: SmtpTransport::relay(smtp_server)
                .unwrap()
                .credentials(Credentials::new(
                    smtp_username.to_string(),
                    smtp_password.to_string(),
                ))
                .build(),
        }
    }

    pub fn send_mail(&self, payload: MailPayload) -> Result<()> {
        self.smtp_transport.send(
            &self
                .message_builder
                .to_owned()
                .to(payload.to.parse()?)
                .subject(payload.subject)
                .body(payload.body)?,
        )?;
        Ok(())
    }
}

pub struct MailPayload {
    to: String,
    subject: String,
    body: String,
}

impl MailPayload {
    pub fn new(to: String, subject: String, body: String) -> Self {
        Self { to, subject, body }
    }
}
