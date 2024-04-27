use anyhow::Result;
use lettre::{
    message::{Mailbox, MessageBuilder},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

pub struct Mailer {
    message_builder: MessageBuilder,
    smtp_transport: SmtpTransport,
    channel_receiver: mpsc::Receiver<MailPayload>,
}

impl Mailer {
    pub fn new(
        smtp_host: &str,
        smtp_username: &str,
        smtp_password: &str,
        sender_name: &str,
        sender_email: &str,
    ) -> (Self, mpsc::Sender<MailPayload>) {
        hb_log::info(Some("⚡"), "Mailer: Initializing component");

        let (sender, receiver) = mpsc::channel::<MailPayload>(32);

        (
            Self {
                message_builder: Message::builder()
                    .from(format!("{sender_name} <{sender_email}>").parse().unwrap()),
                smtp_transport: SmtpTransport::relay(smtp_host)
                    .unwrap()
                    .credentials(Credentials::new(
                        smtp_username.to_owned(),
                        smtp_password.to_owned(),
                    ))
                    .build(),
                channel_receiver: receiver,
            },
            sender,
        )
    }

    pub fn send_mail(&self, payload: &MailPayload) -> Result<()> {
        self.smtp_transport.send(
            &self
                .message_builder
                .to_owned()
                .to(payload.to.parse()?)
                .subject(&payload.subject)
                .body(payload.body.to_string())?,
        )?;
        Ok(())
    }

    pub fn run_none() -> JoinHandle<()> {
        hb_log::info(Some("⏩"), "Mailer: Skipping component");

        tokio::spawn((|| async {})())
    }

    pub fn run(mut self, cancel_token: CancellationToken) -> JoinHandle<()> {
        hb_log::info(Some("💫"), "Mailer: Running component");

        tokio::spawn((|| async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        break;
                    }
                    _ = tokio::signal::ctrl_c() => {
                        break;
                    }
                    recv = self.channel_receiver.recv() => {
                        match recv {
                            Some(payload) => {
                                let mailbox = match payload.to.parse::<Mailbox>() {
                                    Ok(mailbox) => mailbox,
                                    Err(err) => {
                                        hb_log::error(None, &err);
                                        continue;
                                    }
                                };

                                let message = match self
                                    .message_builder
                                    .to_owned()
                                    .to(mailbox)
                                    .subject(payload.subject)
                                    .body(payload.body)
                                {
                                    Ok(message) => message,
                                    Err(err) => {
                                        hb_log::error(None, &err);
                                        continue;
                                    }
                                };

                                if let Err(err) = self.smtp_transport.send(&message) {
                                    hb_log::error(None, &err);
                                }
                            },
                            None => {
                                break;
                            }
                        }
                    }
                }
            }

            hb_log::info(None, "Mailer: Shutting down component");
        })())
    }
}

pub struct MailPayload {
    to: String,
    subject: String,
    body: String,
}

impl MailPayload {
    pub fn new(to: &str, subject: &str, body: &str) -> Self {
        Self {
            to: to.to_owned(),
            subject: subject.to_owned(),
            body: body.to_owned(),
        }
    }
}
